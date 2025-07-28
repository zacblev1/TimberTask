use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::Block;
use ratatui::widgets::Borders;

use crate::app::App;

/// Render the main application layout
#[allow(dead_code)]
pub fn render_layout(f: &mut Frame, _app: &App) {
    let size = f.size();
    
    // Create a layout with two main sections:
    // 1. Timer + current task info (left side)
    // 2. Kanban board (right side)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),  // Timer section
            Constraint::Percentage(60),  // Kanban section
        ])
        .split(size);
    
    // Create blocks for each section
    let timer_block = Block::default()
        .title("Timer")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));
    
    let kanban_block = Block::default()
        .title("Kanban Board")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));
    
    f.render_widget(timer_block, chunks[0]);
    f.render_widget(kanban_block, chunks[1]);
}