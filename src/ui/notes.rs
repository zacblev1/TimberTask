use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Clear};

use crate::app::App;
use crate::state::notes_state::Note;
use crate::utils::text::truncate_text;
use crate::utils::time::format_timestamp;

/// Render the notes tab
pub fn render_notes(f: &mut Frame, app: &App, area: Rect) {
    // Create the layout with sections for notes
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Top bar: Search and tags
            Constraint::Min(0),     // Main content: Notes tree and editor
            Constraint::Length(3),  // Keyboard shortcuts help
        ])
        .margin(1)
        .split(area);
    
    // Render top bar with search and tag filter
    render_top_bar(f, app, chunks[0]);
    
    // Split main area into notes tree and editor
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Notes tree
            Constraint::Percentage(70), // Note editor
        ])
        .split(chunks[1]);
    
    // Render notes tree
    render_notes_tree(f, app, main_chunks[0]);
    
    // Render note editor or note details
    render_note_editor(f, app, main_chunks[1]);
    
    // Render keyboard shortcuts
    render_shortcuts(f, chunks[2]);
    
    // If we're showing the tag form, render it on top
    if app.show_tag_form {
        render_tag_form(f, app, area);
    }
    
    // If we're in note edit mode, render the editor on top
    if app.editing_note {
        render_note_edit_form(f, app, area);
    }
}

/// Render the top bar with search and tags
fn render_top_bar(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("Search & Tags")
        .borders(Borders::ALL);
    
    f.render_widget(block.clone(), area);
    
    let inner_area = block.inner(area);
    
    // Split for search and tags
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Search
            Constraint::Percentage(50), // Tags
        ])
        .split(inner_area);
    
    // Search box
    let search_text = if app.note_search_active {
        format!("Search: {}", app.note_search_query)
    } else {
        "Press '/' to search".to_string()
    };
    
    let search = Paragraph::new(search_text)
        .style(
            if app.note_search_active {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            }
        );
    
    f.render_widget(search, chunks[0]);
    
    // Tag filter display
    let tag_count;
    let active_tag_count = app.active_tag_filters.len();
    
    // Use a temporary scope for the mutex lock
    {
        // Try to lock the mutex, but don't crash if it fails
        if let Ok(notes_state) = app.notes_state.try_lock() {
            tag_count = notes_state.tags.len();
        } else {
            // If we can't get the lock, just show a default message
            tag_count = 0;
        }
    }
    
    let tag_text = if active_tag_count > 0 {
        format!("Tags: {} active filters", active_tag_count)
    } else {
        format!("Tags: {} (press 't' to manage)", tag_count)
    };
    
    let tags = Paragraph::new(tag_text)
        .style(
            if active_tag_count > 0 {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Gray)
            }
        );
    
    f.render_widget(tags, chunks[1]);
}

/// Render the notes tree
fn render_notes_tree(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("Notes")
        .borders(Borders::ALL);
    
    f.render_widget(block.clone(), area);
    
    let inner_area = block.inner(area);
    
    // Try to lock the notes state, but handle the case where we can't get it
    let notes_state_lock = app.notes_state.try_lock();
    let notes_state = match notes_state_lock {
        Ok(notes) => notes,
        Err(_) => {
            // If we can't get the lock, show a message and return
            let error_msg = Paragraph::new("Loading notes...")
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center);
            
            f.render_widget(error_msg, inner_area);
            return;
        }
    };
    
    // If there are no notes, show a message
    if notes_state.notes.is_empty() {
        let empty_msg = Paragraph::new("No notes. Press 'n' to create a new note.")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        
        f.render_widget(empty_msg, inner_area);
        return;
    }
    
    // Determine which notes to display based on search and tags
    let mut display_notes = Vec::new();
    
    if !app.note_search_query.is_empty() {
        // Show search results
        let search_results = notes_state.search_notes(&app.note_search_query);
        for note in search_results {
            display_notes.push((note, 0)); // Flat list for search results
        }
    } else if !app.active_tag_filters.is_empty() {
        // Show notes with the active tags
        for tag_id in &app.active_tag_filters {
            let tagged_notes = notes_state.get_notes_with_tag(tag_id);
            for note in tagged_notes {
                if !display_notes.iter().any(|(n, _)| n.id == note.id) {
                    display_notes.push((note, 0)); // Flat list for tag filters
                }
            }
        }
    } else {
        // Show hierarchical tree
        let root_notes = notes_state.get_root_notes();
        for note in root_notes {
            display_notes.push((note, 0));
            if note.expanded {
                add_child_notes(&notes_state, &mut display_notes, &note.id, 1);
            }
        }
    }
    
    // Create list items
    let items: Vec<ListItem> = display_notes.iter().map(|(note, depth)| {
        // Create indentation based on depth
        let indent = "  ".repeat(*depth);
        
        // Create expansion indicator if the note has children
        let expansion_indicator = if !note.children.is_empty() {
            if note.expanded { "▼ " } else { "▶ " }
        } else {
            "  "
        };
        
        // Truncate text to fit in column width (with padding)
        let width = inner_area.width.saturating_sub(4) as usize;
        let title = truncate_text(&note.title, width - indent.len() - 2);
        
        // Create the display text with indentation
        let display_text = format!("{}{}{}", indent, expansion_indicator, title);
        
        // Check if this note is selected
        let is_selected = notes_state.selected_note_id.as_deref() == Some(&note.id);
        
        if is_selected {
            let style = Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD);
            
            ListItem::new(Span::styled(display_text, style))
        } else {
            let style = Style::default().fg(Color::White);
            ListItem::new(Span::styled(display_text, style))
        }
    }).collect();
    
    let list = List::new(items);
    f.render_widget(list, inner_area);
}

/// Helper function to add child notes to the display list
fn add_child_notes<'a>(notes_state: &'a crate::state::notes_state::NotesState, display_notes: &mut Vec<(&'a Note, usize)>, parent_id: &str, depth: usize) {
    let children = notes_state.get_child_notes(parent_id);
    for child in children {
        display_notes.push((child, depth));
        if child.expanded {
            add_child_notes(notes_state, display_notes, &child.id, depth + 1);
        }
    }
}

/// Render the note editor or detail view
fn render_note_editor(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("Note Details")
        .borders(Borders::ALL);
    
    f.render_widget(block.clone(), area);
    
    let inner_area = block.inner(area);
    
    // Try to lock the notes state
    let notes_state_lock = app.notes_state.try_lock();
    let notes_state = match notes_state_lock {
        Ok(notes) => notes,
        Err(_) => {
            // If we can't get the lock, show a message and return
            let error_msg = Paragraph::new("Loading note details...")
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center);
            
            f.render_widget(error_msg, inner_area);
            return;
        }
    };
    
    // If no note is selected, show a message
    if notes_state.selected_note_id.is_none() {
        let empty_msg = Paragraph::new("No note selected. Select or create a note to view details.")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        
        f.render_widget(empty_msg, inner_area);
        return;
    }
    
    // Get the selected note
    let note = match notes_state.get_selected_note() {
        Some(note) => note,
        None => {
            // This shouldn't happen, but just in case
            let error_msg = Paragraph::new("Error loading selected note.")
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center);
            
            f.render_widget(error_msg, inner_area);
            return;
        }
    };
    
    // Split the area for title, meta info, and content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(2), // Meta info (tags, timestamps)
            Constraint::Min(0),    // Content
        ])
        .split(inner_area);
    
    // Render title
    let title = Paragraph::new(note.title.as_str())
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::BOTTOM));
    
    f.render_widget(title, chunks[0]);
    
    // Render meta info (tags and timestamps)
    let created = format_timestamp(note.created_at);
    let updated = format_timestamp(note.updated_at);
    
    // Format tags
    let tag_str = if note.tags.is_empty() {
        "No tags".to_string()
    } else {
        let tag_names: Vec<String> = note.tags.iter()
            .filter_map(|tag_id| notes_state.get_tag(tag_id))
            .map(|tag| tag.name.clone())
            .collect();
        
        format!("Tags: {}", tag_names.join(", "))
    };
    
    let meta_text = format!("{} | Created: {} | Updated: {}", tag_str, created, updated);
    let meta = Paragraph::new(meta_text)
        .style(Style::default().fg(Color::Gray));
    
    f.render_widget(meta, chunks[1]);
    
    // Render content
    let content = Paragraph::new(note.content.as_str())
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });
    
    f.render_widget(content, chunks[2]);
}

/// Render keyboard shortcuts
fn render_shortcuts(f: &mut Frame, area: Rect) {
    let shortcuts = Paragraph::new(
        "[n] New Note  |  [c] Child Note  |  [Enter] Edit  |  [e] Toggle Expand  |  [d] Delete  |  [t] Manage Tags  |  [/] Search  |  [↑↓←→] Navigate"
    )
    .style(Style::default().fg(Color::White))
    .block(Block::default().borders(Borders::ALL).title("Keyboard Shortcuts"))
    .alignment(Alignment::Center);
    
    f.render_widget(shortcuts, area);
}

/// Render the tag management form
fn render_tag_form(f: &mut Frame, app: &App, area: Rect) {
    // Create a centered box for the form
    let form_area = centered_rect(60, 70, area);
    
    // Clear the background first
    f.render_widget(Clear, form_area);
    
    // Outer block
    let block = Block::default()
        .title("Manage Tags")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));
    
    f.render_widget(block.clone(), form_area);
    
    // Inner area for form content
    let inner_area = block.inner(form_area);
    
    // Split inner area into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(1),  // New tag label
            Constraint::Length(3),  // New tag input
            Constraint::Length(1),  // Spacing
            Constraint::Min(0),     // Tag list
            Constraint::Length(3),  // Buttons
        ])
        .split(inner_area);
    
    // New tag label
    let tag_label = Paragraph::new("New Tag Name:")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left);
    f.render_widget(tag_label, chunks[0]);
    
    // New tag input
    let tag_input = Paragraph::new(app.tag_form_name.as_str())
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).style(
            if app.focused_tag_field == crate::app::TagField::Name {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }
        ));
    f.render_widget(tag_input, chunks[1]);
    
    // Try to get the notes state
    let notes_state_lock = app.notes_state.try_lock();
    match notes_state_lock {
        Err(_) => {
            // If we can't get the lock, show a loading message
            let loading_msg = Paragraph::new("Loading tags...")
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center);
            
            f.render_widget(loading_msg, chunks[3]);
        }
        Ok(notes_state) => {
        let tags: Vec<_> = notes_state.tags.values().collect();
        
        if tags.is_empty() {
            let empty_msg = Paragraph::new("No tags created yet.")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);
            
            f.render_widget(empty_msg, chunks[3]);
        } else {
            let tag_items: Vec<ListItem> = tags.iter().enumerate().map(|(i, tag)| {
                let is_selected = app.selected_tag_idx == Some(i);
                
                let text = format!("{}  {}", if is_selected { "→" } else { " " }, tag.name);
                
                if is_selected {
                    let style = Style::default()
                        .bg(Color::Yellow)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD);
                    
                    ListItem::new(Span::styled(text, style))
                } else {
                    let style = Style::default().fg(Color::White);
                    ListItem::new(Span::styled(text, style))
                }
            }).collect();
            
            let tag_list = List::new(tag_items)
                .block(Block::default().borders(Borders::ALL).title("Tags"));
            
            f.render_widget(tag_list, chunks[3]);
        }
        }
    }
    
    // Buttons
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(chunks[4]);
    
    // Add button
    let add_button = Paragraph::new("[ Add Tag ]")
        .style(
            if app.focused_tag_field == crate::app::TagField::AddButton {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            }
        )
        .alignment(Alignment::Center);
    f.render_widget(add_button, button_chunks[0]);
    
    // Delete button
    let delete_button = Paragraph::new("[ Delete Tag ]")
        .style(
            if app.focused_tag_field == crate::app::TagField::DeleteButton {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            }
        )
        .alignment(Alignment::Center);
    f.render_widget(delete_button, button_chunks[1]);
    
    // Close button
    let close_button = Paragraph::new("[ Close ]")
        .style(
            if app.focused_tag_field == crate::app::TagField::CloseButton {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            }
        )
        .alignment(Alignment::Center);
    f.render_widget(close_button, button_chunks[2]);
    
    // Show cursor for active text field
    if app.show_tag_form && app.focused_tag_field == crate::app::TagField::Name {
        f.set_cursor(
            chunks[1].x + app.tag_form_name.len() as u16 + 1,
            chunks[1].y + 1,
        );
    }
}

/// Render the note edit form
fn render_note_edit_form(f: &mut Frame, app: &App, area: Rect) {
    // Create a centered box for the form
    let form_area = centered_rect(80, 80, area);
    
    // Clear the background first
    f.render_widget(Clear, form_area);
    
    // Outer block
    let block = Block::default()
        .title("Edit Note")
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
            Constraint::Length(1),  // Content label
            Constraint::Min(0),     // Content input - make this larger
            Constraint::Length(1),  // Spacing
            Constraint::Length(3),  // Buttons
        ])
        .split(inner_area);
    
    // Title label
    let title_label = Paragraph::new("Title:")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left);
    f.render_widget(title_label, chunks[0]);
    
    // Title input
    let title_input = Paragraph::new(app.note_form_title.as_str())
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).style(
            if app.focused_note_field == crate::app::NoteField::Title {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }
        ));
    f.render_widget(title_input, chunks[1]);
    
    // Content label
    let content_label = Paragraph::new("Content:")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left);
    f.render_widget(content_label, chunks[3]);
    
    // Content input
    let content_input = Paragraph::new(app.note_form_content.as_str())
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).style(
            if app.focused_note_field == crate::app::NoteField::Content {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }
        ));
    f.render_widget(content_input, chunks[4]);
    
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
            if app.focused_note_field == crate::app::NoteField::CancelButton {
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
            if app.focused_note_field == crate::app::NoteField::SaveButton {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            }
        )
        .alignment(Alignment::Center);
    f.render_widget(save_button, button_chunks[1]);
    
    // Show cursor for active text field
    if app.editing_note {
        match app.focused_note_field {
            crate::app::NoteField::Title => {
                // Position cursor at the end of the title text
                f.set_cursor(
                    chunks[1].x + app.note_form_title.len() as u16 + 1,
                    chunks[1].y + 1,
                );
            }
            crate::app::NoteField::Content => {
                // Position cursor at the end of the content text
                // This is a simplification - in a real app we'd need to handle multiline cursor positioning
                let line_count = app.note_form_content.lines().count().max(1);
                let last_line = app.note_form_content.lines().last().unwrap_or("");
                
                f.set_cursor(
                    chunks[4].x + last_line.len() as u16 + 1,
                    chunks[4].y + (line_count as u16).min(chunks[4].height.saturating_sub(2)),
                );
            }
            _ => {}
        }
    }
}

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