use ratatui::Frame;
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::app::App;
use crate::state::kanban_state::TaskStatus;
use crate::utils::text::truncate_text;

/// Render the kanban board tab
pub fn render_kanban<B: Backend>(f: &mut Frame, app: &App, area: Rect) {
    // Lock kanban state
    let kanban_state = app.kanban_state.lock().unwrap();
    
    // Create the layout with three columns for the kanban board
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Current task and project info
            Constraint::Min(0),     // Kanban columns
            Constraint::Length(3),  // Keyboard shortcuts help
        ])
        .margin(1)
        .split(area);
    
    // Get current task info from timer state
    let timer_state = app.timer_state.lock().unwrap();
    let current_task_id = timer_state.current_task_id.clone();
    drop(timer_state); // Release lock
    
    // Nothing to do here
    
    // Render current task info
    if let Some(task_id) = current_task_id {
        if let Some(task) = kanban_state.get_task(&task_id) {
            let task_info = Paragraph::new(format!("Currently tracking: {} (Time: {})", 
                task.title, 
                format_time_spent(task.time_spent)))
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL).title("Timer Status"));
            f.render_widget(task_info, chunks[0]);
        }
    } else {
        let no_task_info = Paragraph::new("No task selected for tracking - select a task in the \"IN PROGRESS\" column with [Space]")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("Timer Status"));
        f.render_widget(no_task_info, chunks[0]);
    }
    
    // Create the three column layout for kanban
    let board_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(chunks[1]);
    
    // Get project tasks
    let selected_project = kanban_state.get_selected_project();
    
    if let Some(project) = selected_project {
        let tasks = kanban_state.get_project_tasks(&project.id).unwrap_or_default();
        
        // Filter tasks by status
        let todo_tasks: Vec<_> = tasks.iter()
            .filter(|task| task.status == TaskStatus::Todo)
            .collect();
        
        let in_progress_tasks: Vec<_> = tasks.iter()
            .filter(|task| task.status == TaskStatus::InProgress)
            .collect();
        
        let done_tasks: Vec<_> = tasks.iter()
            .filter(|task| task.status == TaskStatus::Done)
            .collect();
        
        // Render todo column
        render_task_column::<B>(f, board_chunks[0], "TODO", &todo_tasks, 0, app.selected_task);
        
        // Render in progress column
        render_task_column::<B>(f, board_chunks[1], "IN PROGRESS", &in_progress_tasks, 1, app.selected_task);
        
        // Render done column
        render_task_column::<B>(f, board_chunks[2], "DONE", &done_tasks, 2, app.selected_task);
    } else {
        // No project selected, show a message
        let no_project_msg = Paragraph::new("No projects found. Create a project with Ctrl+N")
            .style(Style::default().fg(Color::Gray))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(no_project_msg, chunks[1]);
    }
    
    // Render keyboard shortcuts at the bottom with highlighted keys
    let shortcuts = Paragraph::new(
        "[n] New Task  |  [t] To-Do  |  [i] In Progress  |  [d] Done  |  [x] Delete  |  [Space] Track task  |  [↑↓←→] Navigate"
    )
    .style(Style::default().fg(Color::White))
    .block(Block::default().borders(Borders::ALL).title("Keyboard Shortcuts"))
    .alignment(ratatui::layout::Alignment::Center);
    
    f.render_widget(shortcuts, chunks[2]);
}

/// Render a single kanban column with tasks
fn render_task_column<B: Backend>(
    f: &mut Frame, 
    area: Rect, 
    title: &str, 
    tasks: &[&crate::state::kanban_state::Task],
    column_idx: usize,
    selected_task: Option<(usize, usize)>,
) {
    // Create a title with task count and selection status
    let column_title = if let Some((sel_col, _)) = selected_task {
        if sel_col == column_idx {
            format!("{} [{}] ●", title, tasks.len())
        } else {
            format!("{} [{}]", title, tasks.len())
        }
    } else {
        format!("{} [{}]", title, tasks.len())
    };
    
    let block = Block::default()
        .title(column_title)
        .borders(Borders::ALL);
    
    let inner_area = block.inner(area);
    f.render_widget(block, area);
    
    if tasks.is_empty() {
        let empty_msg = Paragraph::new("No tasks")
            .style(Style::default().fg(Color::Gray))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(empty_msg, inner_area);
        return;
    }
    
    // Create a list of task items
    let items: Vec<ListItem> = tasks.iter().enumerate().map(|(i, task)| {
        // Truncate text to fit in column width (with padding)
        let width = inner_area.width.saturating_sub(4) as usize;
        let title = truncate_text(&task.title, width);
        
        // Create task item with title and time spent
        let task_text = format!("{} ({})", title, format_time_spent(task.time_spent));
        
        // Check if this task is selected
        let is_selected = selected_task.map_or(false, |(c, t)| c == column_idx && t == i);
        
        // Make selected item VERY visible
        if is_selected {
            let style = Style::default()
                .bg(Color::Yellow) // Background color
                .fg(Color::Black) // Text color
                .add_modifier(Modifier::BOLD);
                
            let text = format!("→ {}", task_text); // Add an arrow to the selected task
            ListItem::new(Span::styled(text, style))
        } else {
            let style = Style::default().fg(Color::White);
            let text = format!("  {}", task_text); // Add spacing for alignment
            ListItem::new(Span::styled(text, style))
        }
    }).collect();
    
    let list = List::new(items)
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol("→ ");
    
    f.render_widget(list, inner_area);
}

/// Format time spent in a human-readable format
pub fn format_time_spent(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}