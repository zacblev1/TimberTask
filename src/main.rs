use anyhow::{anyhow, Result};
use timber_task::utils::mutex::lock_mutex;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::panic;
use tracing::{error, info, warn};

use timber_task::app::App;
use timber_task::event::{Event, EventHandler};
use timber_task::ui::ui;
use timber_task::logging;

fn main() -> Result<()> {
    // Initialize logging first
    if let Err(e) = logging::init_logging() {
        eprintln!("Failed to initialize logging: {}", e);
        // Continue anyway - logging is not critical for app function
    }
    
    info!("Starting TimberTask application");
    
    // Set up panic hook to restore terminal on crash
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Log the panic
        error!("Application panicked: {:?}", panic_info);
        
        // Try to restore terminal first
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        );
        
        // Then call the original hook
        original_hook(panic_info);
    }));
    
    // Setup terminal
    let terminal_result = setup_terminal();
    
    if let Err(err) = &terminal_result {
        error!("Error setting up terminal: {}", err);
        eprintln!("Error setting up terminal: {}", err);
        eprintln!("This application requires a fully interactive terminal.");
        eprintln!("Please run this program directly in a terminal window, not through an IDE/editor terminal.");
        return Err(anyhow::anyhow!("Terminal setup failed. Please run in a fully interactive terminal."));
    }
    
    let mut terminal = terminal_result?;
    
    
    // Create app state
    info!("Creating application state");
    let mut app = App::new()?;
    
    // Create event handler
    let mut event_handler = EventHandler::new(250);
    
    // Try to select a task if we start on the Kanban tab
    if app.tab_index == 1 {
        info!("Starting on Kanban tab, selecting first available task");
        let _ = app.select_first_available_task();
    }
    
    // Pre-initialize notes if starting on notes tab
    if app.tab_index == 2 {
        info!("Initializing notes tab on startup");
        
        let mut notes_state = lock_mutex(&app.notes_state)?;
        if let Err(e) = notes_state.load_from_disk() {
            error!("Failed to load notes data: {}", e);
        }
        
        // If no note is selected, try to select the first root note
        if notes_state.get_selected_note().is_none() {
            // Get the first root note ID first
            let first_root_id = notes_state.get_root_notes()
                .first()
                .map(|note| note.id.clone());
            
            // Then select it if we found one
            if let Some(id) = first_root_id {
                if let Err(e) = notes_state.select_note(&id) {
                    error!("Failed to select note: {}", e);
                }
            }
        }
    }
    
    // Run the application
    info!("Starting main application loop");
    let res = run_app(&mut terminal, &mut app, &mut event_handler);
    
    // Ensure the event handler is properly shut down
    // This is redundant if run_app completed normally, but ensures cleanup on error
    event_handler.shutdown();
    
    // Save any pending data before exit
    info!("Saving application state before exit");
    if let Ok(kanban) = lock_mutex(&app.kanban_state) {
        if let Err(e) = kanban.save_to_disk() {
            error!("Failed to save kanban state: {}", e);
        }
    }
    if let Ok(notes) = lock_mutex(&app.notes_state) {
        if let Err(e) = notes.save_to_disk() {
            error!("Failed to save notes state: {}", e);
        }
    }
    if let Ok(timer) = lock_mutex(&app.timer_state) {
        if let Err(e) = timer.save_to_disk() {
            error!("Failed to save timer state: {}", e);
        }
    }
    
    // Restore terminal
    restore_terminal(&mut terminal)?;
    
    // Handle any errors that occurred during app execution
    if let Err(err) = res {
        error!("Application error: {:?}", err);
        println!("{:?}", err);
    }
    
    info!("TimberTask application shutting down gracefully");
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    info!("Setting up terminal");
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    
    let execution_result = crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    );
    
    if let Err(e) = execution_result {
        error!("Failed to execute terminal setup commands: {}", e);
        // Make sure to disable raw mode if we fail here
        let _ = crossterm::terminal::disable_raw_mode();
        return Err(anyhow::anyhow!("Failed to execute terminal setup commands: {}", e));
    }
    
    let backend = CrosstermBackend::new(stdout);
    
    match Terminal::new(backend) {
        Ok(terminal) => {
            info!("Terminal setup successful");
            Ok(terminal)
        },
        Err(e) => {
            error!("Failed to create terminal: {}", e);
            // Make sure to clean up if we fail
            let _ = crossterm::terminal::disable_raw_mode();
            Err(anyhow::anyhow!("Failed to create terminal: {}", e))
        }
    }
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    event_handler: &mut EventHandler,
) -> Result<()> {
    // Check initial terminal size
    check_terminal_size(terminal)?;
    
    loop {
        // Draw the UI
        terminal.draw(|f| ui(f, app))?;
        
        // Handle events
        match event_handler.next()? {
            Event::Tick => {
                app.tick();
            }
            Event::Input(key) => {
                if app.handle_key(key)? {
                    // App indicated it should quit
                    break;
                }
            }
            Event::Resize => {
                // Check terminal size after resize
                check_terminal_size(terminal)?;
            }
        }
        
        // Check if app should quit
        if app.should_quit {
            break;
        }
    }
    
    // Shutdown the event handler cleanly
    event_handler.shutdown();
    
    Ok(())
}

fn check_terminal_size(terminal: &Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let size = terminal.size()?;
    
    // Absolute minimum size needed for basic functionality
    if size.width < 80 || size.height < 24 {
        return Err(anyhow!("Terminal too small, min 80x24 required, current {}x{}", 
            size.width, size.height));
    }
    
    // Recommended size for optimal display
    if size.width < 120 || size.height < 30 {
        warn!("Terminal size {}x{} is smaller than recommended 120x30. Some UI elements may not display properly.", 
            size.width, size.height);
        eprintln!("Warning: Terminal size {}x{} is smaller than recommended 120x30. Some UI elements may not display properly.", 
            size.width, size.height);
    }
    
    Ok(())
}