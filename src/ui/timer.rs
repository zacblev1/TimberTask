use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Gauge};

use crate::app::App;
use crate::utils::mutex::lock_mutex;
use crate::utils::time::format_time;
use crate::ui::kanban::format_time_spent;

/// Render the timer tab
pub fn render_timer(f: &mut Frame, app: &App, area: Rect) {
    // Get timer state
    let timer_state = match lock_mutex(&app.timer_state) {
        Ok(state) => state,
        Err(_) => return, // Skip rendering if mutex is poisoned
    };
    
    // Create timer layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Timer period title
            Constraint::Length(3),  // Current task info
            Constraint::Min(12),    // Timer visualization
            Constraint::Length(2),  // Pomodoro count
            Constraint::Length(3),  // Controls
        ])
        .margin(2)
        .split(area);
    
    // Render timer period title
    let period_title = if timer_state.is_work_period { "WORK TIME" } else { "BREAK TIME" };
    let period_color = if timer_state.is_work_period { Color::Red } else { Color::Green };
    
    let title = Paragraph::new(period_title)
        .style(Style::default().fg(period_color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);
    
    // Render current task info
    let current_task_id = timer_state.current_task_id.clone();
    
    // We need to drop the timer_state lock before acquiring the kanban_state lock
    // to avoid potential deadlocks
    drop(timer_state);
    
    let task_info = if let Some(task_id) = current_task_id {
        let kanban_state = match lock_mutex(&app.kanban_state) {
            Ok(state) => state,
            Err(_) => return, // Skip rendering if mutex is poisoned
        };
        if let Some(task) = kanban_state.get_task(&task_id) {
            Paragraph::new(format!("Working on: {} (Total: {})", 
                task.title, 
                format_time_spent(task.time_spent)))
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL).title("Current Task"))
        } else {
            Paragraph::new("No task selected for tracking")
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL).title("Current Task"))
        }
    } else {
        Paragraph::new("No task selected for tracking - Select with [Space] on Kanban tab")
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL).title("Current Task"))
    };
    
    f.render_widget(task_info, chunks[1]);
    
    // Re-acquire timer state
    let timer_state = match lock_mutex(&app.timer_state) {
        Ok(state) => state,
        Err(_) => return, // Skip rendering if mutex is poisoned
    };
    
    // Render timer visualization (circle with progress)
    // For simplicity, we'll use a gauge widget here instead of a custom circle
    let total_seconds = if timer_state.is_work_period {
        timer_state.work_duration.as_secs()
    } else {
        timer_state.break_duration.as_secs()
    };
    
    let remaining_seconds = timer_state.get_remaining_seconds();
    let progress_percent = if total_seconds > 0 {
        (remaining_seconds as f64 / total_seconds as f64) * 100.0
    } else {
        0.0
    };
    
    // Create a formatted time string MM:SS
    let time_str = format_time(remaining_seconds);
    
    // Create a timer visualization using Gauge
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(Style::default().fg(period_color))
        .percent(progress_percent as u16)
        .label(time_str);
    f.render_widget(gauge, chunks[2]);
    
    // Render pomodoro count
    let pomodoro_count = Paragraph::new(format!("Completed Pomodoros: {}", timer_state.completed_pomodoros))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left);
    f.render_widget(pomodoro_count, chunks[3]);
    
    // Render controls
    let controls_text = if timer_state.is_running {
        "[P]ause  [R]eset  [K] Skip & Add Time"
    } else {
        "[S]tart  [R]eset  [T]oggle Work/Break"
    };
    
    let controls = Paragraph::new(controls_text)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);
    f.render_widget(controls, chunks[4]);
}