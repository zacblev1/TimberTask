mod timer;
mod kanban;
mod layout;
mod task_form;
mod task_detail;
mod notes;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, Tabs};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

use crate::app::App;
use timer::render_timer;
use kanban::render_kanban;
use task_form::render_task_form;
use task_detail::render_task_detail;
use notes::render_notes;

/// Render the user interface
pub fn ui(f: &mut Frame, app: &App) {
    // Create main layout (whole screen)
    let size = f.size();
    
    // Create a layout with a tabs row and main content area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
        ])
        .split(size);
    
    // Create tabs
    let tab_titles = ["Timer", "Kanban", "Notes"];
    let titles = tab_titles.iter().map(|t| {
        vec![Span::styled(*t, Style::default().fg(Color::White))]
    }).collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Timber Task"))
        .select(app.tab_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        );
    
    // Render tabs
    f.render_widget(tabs, chunks[0]);
    
    // Render the content based on selected tab
    match app.tab_index {
        0 => render_timer(f, app, chunks[1]),
        1 => render_kanban(f, app, chunks[1]),
        2 => render_notes(f, app, chunks[1]),
        _ => {}
    }
    
    // Render any modals on top
    if app.show_task_form {
        render_task_form(f, app, size);
    }
    
    if app.show_task_detail {
        render_task_detail(f, app, size);
    }
}