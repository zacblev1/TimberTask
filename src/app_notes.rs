use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use std::collections::HashSet;

use crate::app::{App, NoteField, TagField};

impl App {
    /// Handle notes tab key inputs
    pub fn handle_notes_keys(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Navigation
            KeyCode::Up | KeyCode::Char('k') => {
                self.navigate_notes_up()?;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.navigate_notes_down()?;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.navigate_notes_left()?;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.navigate_notes_right()?;
            }
            
            // Note management
            KeyCode::Char('n') => {
                self.open_note_form(None)?;
            }
            KeyCode::Enter => {
                self.edit_selected_note()?;
            }
            KeyCode::Char('e') => {
                self.toggle_note_expanded()?;
            }
            KeyCode::Char('d') => {
                self.delete_selected_note()?;
            }
            KeyCode::Char('c') => {
                self.create_child_note()?;
            }
            
            // Search and tags
            KeyCode::Char('/') => {
                self.start_note_search();
            }
            KeyCode::Char('t') => {
                self.open_tag_form();
            }
            
            _ => {}
        }
        
        Ok(())
    }
    
    /// Handle note search key inputs
    pub fn handle_note_search_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.note_search_active = false;
                self.note_search_query = String::new();
            }
            KeyCode::Enter => {
                self.note_search_active = false;
            }
            KeyCode::Char(c) => {
                self.note_search_query.push(c);
            }
            KeyCode::Backspace => {
                self.note_search_query.pop();
            }
            _ => {}
        }
        
        Ok(false)
    }
    
    /// Handle note edit form key inputs
    pub fn handle_note_edit_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.cancel_note_edit();
            }
            KeyCode::Tab => {
                // Cycle through form fields
                self.focused_note_field = match self.focused_note_field {
                    NoteField::Title => NoteField::Content,
                    NoteField::Content => NoteField::CancelButton,
                    NoteField::CancelButton => NoteField::SaveButton,
                    NoteField::SaveButton => NoteField::Title,
                };
            }
            KeyCode::BackTab => {
                // Cycle through form fields backwards (Shift+Tab)
                self.focused_note_field = match self.focused_note_field {
                    NoteField::Title => NoteField::SaveButton,
                    NoteField::Content => NoteField::Title,
                    NoteField::CancelButton => NoteField::Content,
                    NoteField::SaveButton => NoteField::CancelButton,
                };
            }
            KeyCode::Enter => {
                match self.focused_note_field {
                    NoteField::CancelButton => {
                        self.cancel_note_edit();
                    }
                    NoteField::SaveButton => {
                        self.submit_note_form()?;
                    }
                    // For text fields, move to next field
                    NoteField::Title => {
                        self.focused_note_field = NoteField::Content;
                    }
                    NoteField::Content => {
                        if key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
                            // Shift+Enter adds a newline in content
                            self.note_form_content.push('\n');
                        } else {
                            self.focused_note_field = NoteField::SaveButton;
                        }
                    }
                }
            }
            // Handle text input
            KeyCode::Char(c) => {
                match self.focused_note_field {
                    NoteField::Title => {
                        self.note_form_title.push(c);
                    }
                    NoteField::Content => {
                        self.note_form_content.push(c);
                    }
                    _ => {}
                }
            }
            KeyCode::Backspace => {
                match self.focused_note_field {
                    NoteField::Title => {
                        self.note_form_title.pop();
                    }
                    NoteField::Content => {
                        self.note_form_content.pop();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        
        Ok(false)
    }
    
    /// Handle tag form key inputs
    pub fn handle_tag_form_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.cancel_tag_form();
            }
            KeyCode::Tab => {
                // Cycle through form fields
                self.focused_tag_field = match self.focused_tag_field {
                    TagField::Name => TagField::AddButton,
                    TagField::AddButton => TagField::DeleteButton,
                    TagField::DeleteButton => TagField::CloseButton,
                    TagField::CloseButton => TagField::Name,
                };
            }
            KeyCode::BackTab => {
                // Cycle through form fields backwards (Shift+Tab)
                self.focused_tag_field = match self.focused_tag_field {
                    TagField::Name => TagField::CloseButton,
                    TagField::AddButton => TagField::Name,
                    TagField::DeleteButton => TagField::AddButton,
                    TagField::CloseButton => TagField::DeleteButton,
                };
            }
            KeyCode::Enter => {
                match self.focused_tag_field {
                    TagField::AddButton => {
                        self.add_tag()?;
                    }
                    TagField::DeleteButton => {
                        self.delete_selected_tag()?;
                    }
                    TagField::CloseButton => {
                        self.cancel_tag_form();
                    }
                    TagField::Name => {
                        self.focused_tag_field = TagField::AddButton;
                    }
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.navigate_tag_list_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.navigate_tag_list_down();
            }
            // Handle text input
            KeyCode::Char(c) => {
                if self.focused_tag_field == TagField::Name {
                    self.tag_form_name.push(c);
                }
            }
            KeyCode::Backspace => {
                if self.focused_tag_field == TagField::Name {
                    self.tag_form_name.pop();
                }
            }
            _ => {}
        }
        
        Ok(false)
    }
    
    /// Open the note form for creating or editing a note
    fn open_note_form(&mut self, parent_id: Option<String>) -> Result<()> {
        let notes_state = self.notes_state.lock().unwrap();
        
        // Check if we're editing an existing note
        if let Some(selected_note) = notes_state.get_selected_note() {
            // Editing existing note
            self.note_form_title = selected_note.title.clone();
            self.note_form_content = selected_note.content.clone();
        } else {
            // Creating a new note
            self.note_form_title = String::new();
            self.note_form_content = String::new();
        }
        
        drop(notes_state);
        
        self.focused_note_field = NoteField::Title;
        self.editing_note = true;
        
        Ok(())
    }
    
    /// Cancel note editing and reset form
    fn cancel_note_edit(&mut self) {
        self.note_form_title = String::new();
        self.note_form_content = String::new();
        self.focused_note_field = NoteField::Title;
        self.editing_note = false;
    }
    
    /// Submit the note form to create or update a note
    fn submit_note_form(&mut self) -> Result<()> {
        // Check if title is not empty
        if self.note_form_title.is_empty() {
            // Could show an error message here
            return Ok(());
        }
        
        let mut notes_state = self.notes_state.lock().unwrap();
        
        // Check if we're editing an existing note or creating a new one
        if let Some(selected_note) = notes_state.get_selected_note() {
            // Update existing note
            let note_id = selected_note.id.clone();
            notes_state.update_note(
                &note_id,
                &self.note_form_title,
                &self.note_form_content
            )?;
        } else {
            // Create new note (at root level for now)
            notes_state.create_note(
                &self.note_form_title,
                &self.note_form_content,
                None // Root level note
            )?;
        }
        
        // Reset form and exit edit mode
        self.note_form_title = String::new();
        self.note_form_content = String::new();
        self.focused_note_field = NoteField::Title;
        self.editing_note = false;
        
        Ok(())
    }
    
    /// Edit the currently selected note
    fn edit_selected_note(&mut self) -> Result<()> {
        let notes_state = self.notes_state.lock().unwrap();
        
        if let Some(selected_note) = notes_state.get_selected_note() {
            // Load note data into form
            self.note_form_title = selected_note.title.clone();
            self.note_form_content = selected_note.content.clone();
            
            drop(notes_state);
            
            // Open edit form
            self.focused_note_field = NoteField::Title;
            self.editing_note = true;
        }
        
        Ok(())
    }
    
    /// Toggle expanded state of the selected note
    fn toggle_note_expanded(&mut self) -> Result<()> {
        let mut notes_state = self.notes_state.lock().unwrap();
        
        if let Some(selected_note) = notes_state.get_selected_note() {
            // Only toggle if the note has children
            if !selected_note.children.is_empty() {
                let note_id = selected_note.id.clone();
                notes_state.toggle_note_expanded(&note_id)?;
            }
        }
        
        Ok(())
    }
    
    /// Delete the currently selected note
    fn delete_selected_note(&mut self) -> Result<()> {
        let mut notes_state = self.notes_state.lock().unwrap();
        
        if let Some(selected_note) = notes_state.get_selected_note() {
            let note_id = selected_note.id.clone();
            notes_state.delete_note(&note_id)?;
        }
        
        Ok(())
    }
    
    /// Create a child note under the currently selected note
    fn create_child_note(&mut self) -> Result<()> {
        let notes_state = self.notes_state.lock().unwrap();
        
        // Get parent ID if a note is selected
        let parent_id = if let Some(selected_note) = notes_state.get_selected_note() {
            Some(selected_note.id.clone())
        } else {
            None
        };
        
        drop(notes_state);
        
        // Reset form fields
        self.note_form_title = String::new();
        self.note_form_content = String::new();
        self.focused_note_field = NoteField::Title;
        self.editing_note = true;
        
        // We'll use the parent_id when submitting the form
        if let Some(parent_id) = parent_id {
            let mut notes_state = self.notes_state.lock().unwrap();
            
            // Create a new note with the parent
            notes_state.create_note(
                &self.note_form_title,
                &self.note_form_content,
                Some(&parent_id)
            )?;
        }
        
        Ok(())
    }
    
    /// Start note search
    fn start_note_search(&mut self) {
        self.note_search_active = true;
        self.note_search_query = String::new();
    }
    
    /// Open the tag management form
    fn open_tag_form(&mut self) {
        self.tag_form_name = String::new();
        self.focused_tag_field = TagField::Name;
        self.selected_tag_idx = None;
        self.show_tag_form = true;
    }
    
    /// Cancel tag form and reset fields
    fn cancel_tag_form(&mut self) {
        self.tag_form_name = String::new();
        self.focused_tag_field = TagField::Name;
        self.selected_tag_idx = None;
        self.show_tag_form = false;
    }
    
    /// Add a new tag
    fn add_tag(&mut self) -> Result<()> {
        // Check if name is not empty
        if self.tag_form_name.is_empty() {
            return Ok(());
        }
        
        let mut notes_state = self.notes_state.lock().unwrap();
        notes_state.create_tag(&self.tag_form_name, None)?;
        
        // Reset tag name field
        self.tag_form_name = String::new();
        self.focused_tag_field = TagField::Name;
        
        Ok(())
    }
    
    /// Delete the selected tag
    fn delete_selected_tag(&mut self) -> Result<()> {
        if let Some(idx) = self.selected_tag_idx {
            let mut notes_state = self.notes_state.lock().unwrap();
            
            // Get all tags and find the one at the selected index
            let tags: Vec<_> = notes_state.tags.values().collect();
            
            if idx < tags.len() {
                let tag_id = tags[idx].id.clone();
                notes_state.delete_tag(&tag_id)?;
                
                // Reset selection if we deleted the last tag
                if idx >= notes_state.tags.len() && idx > 0 {
                    self.selected_tag_idx = Some(idx - 1);
                } else if notes_state.tags.is_empty() {
                    self.selected_tag_idx = None;
                }
            }
        }
        
        Ok(())
    }
    
    /// Navigate up in the notes list
    fn navigate_notes_up(&mut self) -> Result<()> {
        // TODO: Implement navigation in the hierarchical notes list
        // This will require keeping track of the visible notes in the UI
        
        Ok(())
    }
    
    /// Navigate down in the notes list
    fn navigate_notes_down(&mut self) -> Result<()> {
        // TODO: Implement navigation in the hierarchical notes list
        
        Ok(())
    }
    
    /// Navigate left in the notes list (collapse)
    fn navigate_notes_left(&mut self) -> Result<()> {
        let mut notes_state = self.notes_state.lock().unwrap();
        
        if let Some(selected_note) = notes_state.get_selected_note() {
            // If the note has children and is expanded, collapse it
            if !selected_note.children.is_empty() && selected_note.expanded {
                notes_state.toggle_note_expanded(&selected_note.id)?;
            } 
            // If the note has a parent, go to the parent
            else if let Some(parent_id) = &selected_note.parent_id {
                notes_state.select_note(parent_id)?;
            }
        }
        
        Ok(())
    }
    
    /// Navigate right in the notes list (expand)
    fn navigate_notes_right(&mut self) -> Result<()> {
        let mut notes_state = self.notes_state.lock().unwrap();
        
        if let Some(selected_note) = notes_state.get_selected_note() {
            // If the note has children
            if !selected_note.children.is_empty() {
                let note_id = selected_note.id.clone();
                
                // If not expanded, expand it
                if !selected_note.expanded {
                    notes_state.toggle_note_expanded(&note_id)?;
                } 
                // If already expanded, go to first child
                else if !selected_note.children.is_empty() {
                    let first_child_id = selected_note.children[0].clone();
                    notes_state.select_note(&first_child_id)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Navigate up in the tag list
    fn navigate_tag_list_up(&mut self) {
        let notes_state = self.notes_state.lock().unwrap();
        let tag_count = notes_state.tags.len();
        
        if tag_count == 0 {
            self.selected_tag_idx = None;
            return;
        }
        
        if let Some(idx) = self.selected_tag_idx {
            if idx > 0 {
                self.selected_tag_idx = Some(idx - 1);
            }
        } else {
            self.selected_tag_idx = Some(tag_count - 1);
        }
    }
    
    /// Navigate down in the tag list
    fn navigate_tag_list_down(&mut self) {
        let notes_state = self.notes_state.lock().unwrap();
        let tag_count = notes_state.tags.len();
        
        if tag_count == 0 {
            self.selected_tag_idx = None;
            return;
        }
        
        if let Some(idx) = self.selected_tag_idx {
            if idx < tag_count - 1 {
                self.selected_tag_idx = Some(idx + 1);
            }
        } else {
            self.selected_tag_idx = Some(0);
        }
    }
}