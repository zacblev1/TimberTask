use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Clear, Wrap};
use chrono::{Utc, TimeZone};

use crate::app::App;
use crate::utils::mutex::lock_mutex;
use super::task_form::centered_rect;
use super::kanban::format_time_spent;

/// Render the task detail view
pub fn render_task_detail(f: &mut Frame, app: &App, area: Rect) {
    // Get the selected task
    let selected_task_info = if let Some((col, row)) = app.selected_task {
        let kanban_state = match lock_mutex(&app.kanban_state) {
            Ok(state) => state,
            Err(_) => return, // Skip rendering if mutex is poisoned
        };
        
        if let Some(project) = kanban_state.get_selected_project() {
            let tasks = kanban_state.get_project_tasks(&project.id).unwrap_or_default();
            
            let target_status = match col {
                0 => crate::state::kanban_state::TaskStatus::Todo,
                1 => crate::state::kanban_state::TaskStatus::InProgress,
                2 => crate::state::kanban_state::TaskStatus::Done,
                _ => return,
            };
            
            let tasks_in_column: Vec<_> = tasks
                .iter()
                .filter(|t| t.status == target_status)
                .collect();
            
            tasks_in_column.get(row).map(|t| {
                let created_dt = Utc.timestamp_opt(t.created_at as i64, 0).unwrap();
                let updated_dt = Utc.timestamp_opt(t.updated_at as i64, 0).unwrap();
                
                (
                    t.title.clone(),
                    t.description.clone(),
                    format!("{:?}", t.status),
                    format_time_spent(t.time_spent),
                    created_dt.format("%Y-%m-%d %H:%M:%S").to_string(),
                    updated_dt.format("%Y-%m-%d %H:%M:%S").to_string(),
                )
            })
        } else {
            None
        }
    } else {
        None
    };
    
    // If no task is selected, don't render anything
    let (title, description, status, time_spent, created_at, updated_at) = match selected_task_info {
        Some(info) => info,
        None => return,
    };
    
    // Create a centered box for the detail view - make it larger
    let detail_area = centered_rect(80, 70, area);
    
    // Clear the background first
    f.render_widget(Clear, detail_area);
    
    // Outer block
    let block = Block::default()
        .title(" Task Details ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));
    
    f.render_widget(block.clone(), detail_area);
    
    // Inner area for content
    let inner_area = block.inner(detail_area);
    
    // Split inner area into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),   // Title
            Constraint::Length(1),   // Separator
            Constraint::Min(8),      // Description
            Constraint::Length(1),   // Separator
            Constraint::Length(2),   // Status
            Constraint::Length(2),   // Time spent
            Constraint::Length(2),   // Created at
            Constraint::Length(2),   // Updated at
            Constraint::Length(1),   // Separator
            Constraint::Length(3),   // Instructions
        ])
        .split(inner_area);
    
    // Title
    let title_widget = Paragraph::new(title)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .block(Block::default().title("Title").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(title_widget, chunks[0]);
    
    // Description
    let desc_text = if description.is_empty() {
        "(No description)".to_string()
    } else {
        description
    };
    let desc_widget = Paragraph::new(desc_text)
        .style(Style::default().fg(Color::White))
        .block(Block::default().title("Description").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(desc_widget, chunks[2]);
    
    // Status
    let status_widget = Paragraph::new(format!("Status: {}", status))
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(status_widget, chunks[4]);
    
    // Time spent
    let time_widget = Paragraph::new(format!("Time Spent: {}", time_spent))
        .style(Style::default().fg(Color::Green));
    f.render_widget(time_widget, chunks[5]);
    
    // Created at
    let created_widget = Paragraph::new(format!("Created: {}", created_at))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(created_widget, chunks[6]);
    
    // Updated at
    let updated_widget = Paragraph::new(format!("Updated: {}", updated_at))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(updated_widget, chunks[7]);
    
    // Instructions
    let instructions = Paragraph::new("Press [Esc], [q], or [v] to close")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(instructions, chunks[9]);
}