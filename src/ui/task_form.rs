use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Clear, Wrap};

use crate::app::{App, FormField};

/// Helper function to create a centered rect using a percentage of the available rect
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Render the task creation form
pub fn render_task_form(f: &mut Frame, app: &App, area: Rect) {
    // Create a centered box for the form - make it larger
    let form_area = centered_rect(80, 60, area);
    
    // Clear the background first
    f.render_widget(Clear, form_area);
    
    // Outer block
    let block = Block::default()
        .title("New Task")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));
    
    f.render_widget(block.clone(), form_area);
    
    // Inner area for form content
    let inner_area = block.inner(form_area);
    
    // Split inner area into sections for each form field
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(1),  // Title label
            Constraint::Length(3),  // Title input
            Constraint::Length(1),  // Spacing
            Constraint::Length(1),  // Description label
            Constraint::Length(8),  // Description input - make this larger
            Constraint::Length(1),  // Spacing
            Constraint::Length(3),  // Buttons
        ])
        .split(inner_area);
    
    // Title label
    let title_label = Paragraph::new("Title:")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left);
    f.render_widget(title_label, chunks[0]);
    
    // Title input with text wrapping
    let title_input = Paragraph::new(app.task_form_title.as_str())
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).style(
            if app.focused_field == FormField::Title {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }
        ))
        .wrap(Wrap { trim: true });
    f.render_widget(title_input, chunks[1]);
    
    // Description label
    let desc_label = Paragraph::new("Description:")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left);
    f.render_widget(desc_label, chunks[3]);
    
    // Description input with text wrapping
    let desc_input = Paragraph::new(app.task_form_description.as_str())
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).style(
            if app.focused_field == FormField::Description {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }
        ))
        .wrap(Wrap { trim: true });
    f.render_widget(desc_input, chunks[4]);
    
    // Buttons
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(chunks[6]);
    
    // Cancel button
    let cancel_button = Paragraph::new("[ Cancel ]")
        .style(
            if app.focused_field == FormField::CancelButton {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            }
        )
        .alignment(Alignment::Center);
    f.render_widget(cancel_button, button_chunks[0]);
    
    // Save button
    let save_button = Paragraph::new("[ Save ]")
        .style(
            if app.focused_field == FormField::SaveButton {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            }
        )
        .alignment(Alignment::Center);
    f.render_widget(save_button, button_chunks[1]);
    
    // Show cursor for active text field
    if app.show_task_form {
        match app.focused_field {
            FormField::Title => {
                // For wrapped text, we need to calculate the cursor position differently
                // This is a simplified approach - for full support, we'd need to calculate line breaks
                let text_len = app.task_form_title.len() as u16;
                let field_width = chunks[1].width.saturating_sub(2); // Account for borders
                
                if text_len < field_width {
                    // Text fits on one line
                    f.set_cursor(
                        chunks[1].x + text_len + 1,
                        chunks[1].y + 1,
                    );
                }
                // For multi-line text, cursor positioning would need more complex calculation
            }
            FormField::Description => {
                // Similar approach for description
                let text_len = app.task_form_description.len() as u16;
                let field_width = chunks[4].width.saturating_sub(2); // Account for borders
                
                if text_len < field_width {
                    // Text fits on one line
                    f.set_cursor(
                        chunks[4].x + text_len + 1,
                        chunks[4].y + 1,
                    );
                }
                // For multi-line text, cursor positioning would need more complex calculation
            }
            _ => {}
        }
    }
}