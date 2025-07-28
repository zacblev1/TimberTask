use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use super::App;
use crate::utils::mutex::lock_mutex;

impl App {
    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        // Handle forms/modals first if they're open
        if self.show_task_detail {
            return self.handle_task_detail_keys(key);
        }
        
        if self.show_task_form {
            return self.handle_task_form_keys(key);
        }
        
        if self.editing_note {
            return self.handle_note_edit_keys(key);
        }
        
        if self.show_tag_form {
            return self.handle_tag_form_keys(key);
        }
        
        if self.note_search_active {
            return self.handle_note_search_keys(key);
        }
        
        // Global keys (work across all tabs)
        match key.code {
            KeyCode::Char('q') => {
                if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                    self.should_quit = true;
                    return Ok(true);
                }
            }
            KeyCode::Tab => {
                if key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
                    // Shift+Tab - go backward
                    if self.tab_index == 0 {
                        self.tab_index = 2;
                    } else {
                        self.tab_index -= 1;
                    }
                } else {
                    // Tab - go forward
                    self.tab_index = (self.tab_index + 1) % 3;
                }
                
                // Try to select a task when switching to Kanban tab
                if self.tab_index == 1 {
                    tracing::debug!("Switched to Kanban tab, attempting to select first available task");
                    match self.select_first_available_task() {
                        Ok(_) => tracing::debug!("Selected task: {:?}", self.selected_task),
                        Err(e) => tracing::error!("Failed to select first task: {}", e),
                    }
                }
                
                // Ensure notes are loaded when switching to Notes tab
                if self.tab_index == 2 {
                    let mut notes_state = lock_mutex(&self.notes_state)?;
                    
                    // If no note is selected, try to select the first root note
                    if notes_state.get_selected_note().is_none() {
                        let first_root_id = notes_state.get_root_notes()
                            .first()
                            .map(|note| note.id.clone());
                        
                        if let Some(id) = first_root_id {
                            let _ = notes_state.select_note(&id);
                        }
                    }
                }
                
                return Ok(false);
            }
            _ => {}
        }
        
        // Tab-specific keys
        match self.tab_index {
            0 => self.handle_timer_keys(key),
            1 => self.handle_kanban_keys(key),
            2 => self.handle_notes_keys(key),
            _ => Ok(false),
        }
    }
}