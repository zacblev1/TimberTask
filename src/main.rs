use anyhow::{anyhow, Result};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::fs;
use std::io;
use std::panic;

mod app;
mod debug;
mod event;
mod state;
mod ui;
mod utils;

use app::App;
use event::{Event, EventHandler};
use ui::ui;

fn main() -> Result<()> {
    // Set up panic hook to restore terminal on crash
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
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
        eprintln!("Error setting up terminal: {}", err);
        eprintln!("This application requires a fully interactive terminal.");
        eprintln!("Please run this program directly in a terminal window, not through an IDE/editor terminal.");
        return Err(anyhow::anyhow!("Terminal setup failed. Please run in a fully interactive terminal."));
    }
    
    let mut terminal = terminal_result?;
    
    // Set up logging
    let log_file = home::home_dir()
        .expect("Failed to get home directory")
        .join(".timber-task")
        .join("app.log");
    
    // Create log directory if it doesn't exist
    if let Some(parent) = log_file.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).unwrap_or_else(|e| {
                eprintln!("Failed to create log directory: {}", e);
            });
        }
    }
    
    // Create app state
    let mut app = App::new()?;
    
    // Create event handler
    let mut event_handler = EventHandler::new(250);
    
    // Try to select a task if we start on the Kanban tab
    if app.tab_index == 1 {
        let _ = app.select_first_available_task();
    }
    
    // Pre-initialize notes if starting on notes tab
    if app.tab_index == 2 {
        // Log this attempt
        fs::write(&log_file, "Initializing notes tab on startup\n").unwrap_or_else(|e| {
            eprintln!("Failed to write to log file: {}", e);
        });
        
        let mut notes_state = app.notes_state.lock().unwrap();
        if let Err(e) = notes_state.load_from_disk() {
            let error_msg = format!("Failed to load notes data: {}\n", e);
            fs::write(&log_file, error_msg).unwrap_or_else(|e| {
                eprintln!("Failed to write to log file: {}", e);
            });
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
                    let error_msg = format!("Failed to select note: {}\n", e);
                    fs::write(&log_file, error_msg).unwrap_or_else(|e| {
                        eprintln!("Failed to write to log file: {}", e);
                    });
                }
            }
        }
    }
    
    // Run the application
    let res = run_app(&mut terminal, &mut app, &mut event_handler);
    
    // Restore terminal
    restore_terminal(&mut terminal)?;
    
    // Handle any errors that occurred during app execution
    if let Err(err) = res {
        println!("{:?}", err);
    }
    
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    
    let execution_result = crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    );
    
    if let Err(e) = execution_result {
        // Make sure to disable raw mode if we fail here
        let _ = crossterm::terminal::disable_raw_mode();
        return Err(anyhow::anyhow!("Failed to execute terminal setup commands: {}", e));
    }
    
    let backend = CrosstermBackend::new(stdout);
    
    match Terminal::new(backend) {
        Ok(terminal) => Ok(terminal),
        Err(e) => {
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
        terminal.draw(|f| ui::<CrosstermBackend<io::Stdout>>(f, app))?;
        
        // Handle events
        match event_handler.next()? {
            Event::Tick => {
                app.tick();
            }
            Event::Input(key) => {
                if app.handle_key(key)? {
                    return Ok(());
                }
            }
            Event::Resize => {
                // Check terminal size after resize
                check_terminal_size(terminal)?;
            }
        }
    }
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
        eprintln!("Warning: Terminal size {}x{} is smaller than recommended 120x30. Some UI elements may not display properly.", 
            size.width, size.height);
    }
    
    Ok(())
}