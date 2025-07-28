use anyhow::{Result, anyhow};
use crossterm::event::{KeyCode, KeyEvent};
use std::collections::HashSet;
use std::fs;
use std::sync::{Arc, Mutex};

use crate::utils::mutex::lock_mutex;

use crate::state::{
    kanban_state::{KanbanState, TaskStatus},
    timer_state::TimerState,
    notes_state::NotesState,
};

/// Form field focus options for task modal dialogs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormField {
    Title,
    Description,
    CancelButton,
    SaveButton,
}

/// Field focus options for the note editor
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteField {
    Title,
    Content,
    CancelButton,
    SaveButton,
}

/// Field focus options for the tag management form
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TagField {
    Name,
    AddButton,
    DeleteButton,
    CloseButton,
}

/// Application state and logic
// Notes functionality is now implemented directly in this file

// Helper enum for left note navigation actions
enum LeftAction {
    Collapse(String),
    GoToParent(String),
}

// Helper enum for right navigation actions
enum RightAction {
    Expand(String),
    GoToFirstChild(String),
}

pub struct App {
    /// Timer state
    pub timer_state: Arc<Mutex<TimerState>>,
    /// Kanban board state
    pub kanban_state: Arc<Mutex<KanbanState>>,
    /// Notes state
    pub notes_state: Arc<Mutex<NotesState>>,
    /// Current active tab index
    pub tab_index: usize,
    /// Whether settings dialog is open
    pub show_settings: bool,
    /// Whether help dialog is open
    pub show_help: bool,
    /// Whether a task form is open
    pub show_task_form: bool,
    /// Currently focused task form field
    pub focused_field: FormField,
    /// Task form title input
    pub task_form_title: String,
    /// Task form description input
    pub task_form_description: String,
    /// Selected task in Kanban board
    pub selected_task: Option<(usize, usize)>,
    /// Whether app should quit
    pub should_quit: bool,
    
    // Notes-related fields
    /// Whether a note is being edited
    pub editing_note: bool,
    /// Whether we're editing an existing note (true) or creating a new one (false)
    pub is_editing_existing_note: bool,
    /// Currently focused note field
    pub focused_note_field: NoteField,
    /// Note form title input
    pub note_form_title: String,
    /// Note form content input
    pub note_form_content: String,
    /// Whether search is active in notes
    pub note_search_active: bool,
    /// Current search query for notes
    pub note_search_query: String,
    /// Parent note ID when creating a child note
    pub parent_note_id: Option<String>,
    
    // Tag-related fields
    /// Whether tag form is open
    pub show_tag_form: bool,
    /// Currently focused tag field
    pub focused_tag_field: TagField,
    /// Tag form name input
    pub tag_form_name: String,
    /// Selected tag index in the tag list
    pub selected_tag_idx: Option<usize>,
    /// Active tag filters for notes
    pub active_tag_filters: HashSet<String>,
}

impl App {
    // Note-related methods
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
            KeyCode::Tab | KeyCode::Char('\t') => {
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
            KeyCode::Tab | KeyCode::Char('\t') => {
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
    
    /// Open the note form for creating a new note
    fn open_note_form(&mut self, parent_id: Option<String>) -> Result<()> {
        // We're creating a new note, not editing an existing one
        self.is_editing_existing_note = false;
        
        // Creating a new note
        self.note_form_title = String::new();
        self.note_form_content = String::new();
        
        // Store the parent ID if provided
        self.parent_note_id = parent_id;
        
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
        self.is_editing_existing_note = false; // Reset editing flag
        self.parent_note_id = None; // Also reset parent_note_id
    }
    
    /// Submit the note form to create or update a note
    fn submit_note_form(&mut self) -> Result<()> {
        // Check if title is not empty
        if self.note_form_title.is_empty() {
            // Could show an error message here
            return Ok(());
        }
        
        let mut notes_state = lock_mutex(&self.notes_state)?;
        
        // Check if we're editing an existing note or creating a new one
        if self.is_editing_existing_note {
            // We're editing an existing note
            if let Some(selected_note) = notes_state.get_selected_note() {
                // Update existing note
                let note_id = selected_note.id.clone();
                notes_state.update_note(
                    &note_id,
                    &self.note_form_title,
                    &self.note_form_content
                )?;
            }
        } else {
            // We're creating a new note (either as child or root level)
            notes_state.create_note(
                &self.note_form_title,
                &self.note_form_content,
                self.parent_note_id.as_deref() // Use parent_note_id if available
            )?;
        }
        
        // Explicitly save changes to disk
        notes_state.save_to_disk()?;
        
        // Reset form and exit edit mode
        self.note_form_title = String::new();
        self.note_form_content = String::new();
        self.focused_note_field = NoteField::Title;
        self.editing_note = false;
        self.is_editing_existing_note = false; // Reset editing flag
        self.parent_note_id = None; // Reset parent_note_id
        
        Ok(())
    }
    
    /// Edit the currently selected note
    fn edit_selected_note(&mut self) -> Result<()> {
        let notes_state = lock_mutex(&self.notes_state)?;
        
        if let Some(selected_note) = notes_state.get_selected_note() {
            // Load note data into form
            self.note_form_title = selected_note.title.clone();
            self.note_form_content = selected_note.content.clone();
            
            drop(notes_state);
            
            // We're editing an existing note, not creating a new one
            self.is_editing_existing_note = true;
            self.parent_note_id = None; // When editing, we don't need parent_note_id
            
            // Open edit form
            self.focused_note_field = NoteField::Title;
            self.editing_note = true;
        }
        
        Ok(())
    }
    
    /// Toggle expanded state of the selected note
    fn toggle_note_expanded(&mut self) -> Result<()> {
        let mut notes_state = lock_mutex(&self.notes_state)?;
        
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
        let mut notes_state = lock_mutex(&self.notes_state)?;
        
        if let Some(selected_note) = notes_state.get_selected_note() {
            let note_id = selected_note.id.clone();
            notes_state.delete_note(&note_id)?;
        }
        
        Ok(())
    }
    
    /// Create a child note under the currently selected note
    fn create_child_note(&mut self) -> Result<()> {
        let notes_state = lock_mutex(&self.notes_state)?;
        
        // Get parent ID if a note is selected
        let parent_id = if let Some(selected_note) = notes_state.get_selected_note() {
            Some(selected_note.id.clone())
        } else {
            None
        };
        
        drop(notes_state);
        
        // We're creating a new note, not editing an existing one
        self.is_editing_existing_note = false;
        
        // Store the parent_id for later use during form submission
        self.parent_note_id = parent_id;
        
        // Reset form fields and open the editor
        self.note_form_title = String::new();
        self.note_form_content = String::new();
        self.focused_note_field = NoteField::Title;
        self.editing_note = true;
        
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
        
        let mut notes_state = lock_mutex(&self.notes_state)?;
        notes_state.create_tag(&self.tag_form_name, None)?;
        
        // Reset tag name field
        self.tag_form_name = String::new();
        self.focused_tag_field = TagField::Name;
        
        Ok(())
    }
    
    /// Delete the selected tag
    fn delete_selected_tag(&mut self) -> Result<()> {
        if let Some(idx) = self.selected_tag_idx {
            let mut notes_state = lock_mutex(&self.notes_state)?;
            
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
        let mut notes_state = lock_mutex(&self.notes_state)?;
        
        // Get all visible notes in the current view
        let mut visible_notes = Vec::new();
        
        if !self.note_search_query.is_empty() {
            // Show search results
            let search_results = notes_state.search_notes(&self.note_search_query);
            for note in search_results {
                visible_notes.push(note.id.clone());
            }
        } else if !self.active_tag_filters.is_empty() {
            // Show notes with the active tags
            for tag_id in &self.active_tag_filters {
                let tagged_notes = notes_state.get_notes_with_tag(tag_id);
                for note in tagged_notes {
                    if !visible_notes.contains(&note.id) {
                        visible_notes.push(note.id.clone());
                    }
                }
            }
        } else {
            // Show hierarchical tree
            let root_notes = notes_state.get_root_notes();
            for note in root_notes {
                visible_notes.push(note.id.clone());
                if note.expanded {
                    add_visible_child_notes(&notes_state, &mut visible_notes, &note.id);
                }
            }
        }
        
        // Find current selection index
        let current_idx = if let Some(selected_id) = &notes_state.selected_note_id {
            visible_notes.iter().position(|id| id == selected_id)
        } else {
            None
        };
        
        // Select previous note if possible
        if let Some(idx) = current_idx {
            if idx > 0 {
                let prev_id = &visible_notes[idx - 1];
                notes_state.select_note(prev_id)?;
            }
        } else if !visible_notes.is_empty() {
            // If nothing selected, select the last note
            let last_id = &visible_notes[visible_notes.len() - 1];
            notes_state.select_note(last_id)?;
        }
        
        Ok(())
    }
    
    /// Navigate down in the notes list
    fn navigate_notes_down(&mut self) -> Result<()> {
        let mut notes_state = lock_mutex(&self.notes_state)?;
        
        // Get all visible notes in the current view
        let mut visible_notes = Vec::new();
        
        if !self.note_search_query.is_empty() {
            // Show search results
            let search_results = notes_state.search_notes(&self.note_search_query);
            for note in search_results {
                visible_notes.push(note.id.clone());
            }
        } else if !self.active_tag_filters.is_empty() {
            // Show notes with the active tags
            for tag_id in &self.active_tag_filters {
                let tagged_notes = notes_state.get_notes_with_tag(tag_id);
                for note in tagged_notes {
                    if !visible_notes.contains(&note.id) {
                        visible_notes.push(note.id.clone());
                    }
                }
            }
        } else {
            // Show hierarchical tree
            let root_notes = notes_state.get_root_notes();
            for note in root_notes {
                visible_notes.push(note.id.clone());
                if note.expanded {
                    add_visible_child_notes(&notes_state, &mut visible_notes, &note.id);
                }
            }
        }
        
        // Find current selection index
        let current_idx = if let Some(selected_id) = &notes_state.selected_note_id {
            visible_notes.iter().position(|id| id == selected_id)
        } else {
            None
        };
        
        // Select next note if possible
        if let Some(idx) = current_idx {
            if idx < visible_notes.len() - 1 {
                let next_id = &visible_notes[idx + 1];
                notes_state.select_note(next_id)?;
            }
        } else if !visible_notes.is_empty() {
            // If nothing selected, select the first note
            let first_id = &visible_notes[0];
            notes_state.select_note(first_id)?;
        }
        
        Ok(())
    }
    
    /// Navigate left in the notes list (collapse)
    fn navigate_notes_left(&mut self) -> Result<()> {
        let mut notes_state = lock_mutex(&self.notes_state)?;
        
        // Clone necessary data to avoid borrow conflicts
        let action = if let Some(selected_note) = notes_state.get_selected_note() {
            // If the note has children and is expanded, collapse it
            if !selected_note.children.is_empty() && selected_note.expanded {
                let note_id = selected_note.id.clone();
                Some(LeftAction::Collapse(note_id))
            } 
            // If the note has a parent, go to the parent
            else if let Some(parent_id) = &selected_note.parent_id {
                let parent_id = parent_id.clone();
                Some(LeftAction::GoToParent(parent_id))
            } else {
                None
            }
        } else {
            None
        };
        
        // Apply the action outside the borrow
        if let Some(action) = action {
            match action {
                LeftAction::Collapse(note_id) => {
                    notes_state.toggle_note_expanded(&note_id)?;
                },
                LeftAction::GoToParent(parent_id) => {
                    notes_state.select_note(&parent_id)?;
                }
            }
        }
        
        Ok(())
    }
    
    // Navigation helpers defined at module level
    
    /// Navigate right in the notes list (expand)
    fn navigate_notes_right(&mut self) -> Result<()> {
        let mut notes_state = lock_mutex(&self.notes_state)?;
        
        // Clone necessary data to avoid borrow conflicts
        let action = if let Some(selected_note) = notes_state.get_selected_note() {
            // If the note has children
            if !selected_note.children.is_empty() {
                let note_id = selected_note.id.clone();
                
                // If not expanded, expand it
                if !selected_note.expanded {
                    Some(RightAction::Expand(note_id))
                } 
                // If already expanded, go to first child
                else if !selected_note.children.is_empty() {
                    let first_child_id = selected_note.children[0].clone();
                    Some(RightAction::GoToFirstChild(first_child_id))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        
        // Apply the action outside the borrow
        if let Some(action) = action {
            match action {
                RightAction::Expand(note_id) => {
                    notes_state.toggle_note_expanded(&note_id)?;
                },
                RightAction::GoToFirstChild(child_id) => {
                    notes_state.select_note(&child_id)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Navigate up in the tag list
    fn navigate_tag_list_up(&mut self) {
        let notes_state = match lock_mutex(&self.notes_state) {
            Ok(state) => state,
            Err(_) => return, // Skip if mutex is poisoned
        };
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
        let notes_state = match lock_mutex(&self.notes_state) {
            Ok(state) => state,
            Err(_) => return, // Skip if mutex is poisoned
        };
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
    
    /// Create a new application instance
    pub fn new() -> Result<Self> {
        // Set up logging
        let log_file = home::home_dir()
            .ok_or_else(|| anyhow!("Failed to get home directory"))?
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
        
        // Initialize app state
        let timer_state = Arc::new(Mutex::new(TimerState::default()));
        let kanban_state = Arc::new(Mutex::new(KanbanState::default()));
        let notes_state = Arc::new(Mutex::new(NotesState::default()));
        
        // Log initialization
        let init_msg = "Initializing app state\n";
        fs::write(&log_file, init_msg).unwrap_or_else(|e| {
            eprintln!("Failed to write to log file: {}", e);
        });
        
        // Load saved data
        {
            let mut kanban = lock_mutex(&kanban_state)?;
            if let Err(e) = kanban.load_from_disk() {
                let err_msg = format!("Warning: Failed to load kanban data: {}\n", e);
                fs::write(&log_file, &err_msg).unwrap_or_else(|_| {
                    eprintln!("{}", err_msg);
                });
            } else {
                fs::write(&log_file, "Successfully loaded kanban data\n").unwrap_or_else(|_| {});
            }
        }
        
        // Load notes data
        {
            let mut notes = lock_mutex(&notes_state)?;
            if let Err(e) = notes.load_from_disk() {
                let err_msg = format!("Warning: Failed to load notes data: {}\n", e);
                fs::write(&log_file, &err_msg).unwrap_or_else(|_| {
                    eprintln!("{}", err_msg);
                });
            } else {
                fs::write(&log_file, "Successfully loaded notes data\n").unwrap_or_else(|_| {});
            }
            
            // We no longer create a welcome note by default
        }
        
        Ok(Self {
            timer_state,
            kanban_state,
            notes_state,
            tab_index: 0,
            show_settings: false,
            show_help: false,
            show_task_form: false,
            focused_field: FormField::Title,
            task_form_title: String::new(),
            task_form_description: String::new(),
            selected_task: None,
            should_quit: false,
            
            // Initialize notes-related fields
            editing_note: false,
            is_editing_existing_note: false,
            focused_note_field: NoteField::Title,
            note_form_title: String::new(),
            note_form_content: String::new(),
            note_search_active: false,
            note_search_query: String::new(),
            parent_note_id: None,
            
            // Initialize tag-related fields
            show_tag_form: false,
            focused_tag_field: TagField::Name,
            tag_form_name: String::new(),
            selected_tag_idx: None,
            active_tag_filters: HashSet::new(),
        })
    }
    
    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        // Handle forms/modals first if they're open
        if self.show_task_form {
            return self.handle_task_form_keys(key);
        }
        
        if self.editing_note {
            return self.handle_note_edit_keys(key);
        }
        
        if self.show_tag_form {
            return self.handle_tag_form_keys(key);
        }
        
        // Handle note search if active
        if self.note_search_active && self.tab_index == 2 {
            return self.handle_note_search_keys(key);
        }
        
        match key.code {
            // Global shortcuts
            KeyCode::F(10) | KeyCode::Char('q') => {
                self.should_quit = true;
                return Ok(true);
            }
            // Tab key handling - check for both KeyCode::Tab and Char('\t') for terminal compatibility
            KeyCode::Tab | KeyCode::Char('\t') => {
                if key.modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
                    // Cycle backwards through tabs with Shift+Tab
                    self.tab_index = (self.tab_index + 2) % 3; // +2 is the same as -1 with modulo 3
                } else {
                    // Cycle forwards through tabs
                    self.tab_index = (self.tab_index + 1) % 3;
                }
                
                // Handle tab-specific initialization
                match self.tab_index {
                    1 => {
                        // If switching to kanban tab and no task is selected, try to select one
                        if self.selected_task.is_none() {
                            self.select_first_available_task()?;
                        }
                    },
                    2 => {
                        // If switching to notes tab, ensure we have a selected note
                        {
                            let mut notes_state = lock_mutex(&self.notes_state)?;
                        
                            // Make sure notes are loaded from disk
                            if notes_state.notes.is_empty() {
                                if let Err(e) = notes_state.load_from_disk() {
                                    eprintln!("Warning: Failed to load notes data: {}", e);
                                    
                                    // We no longer create a welcome note automatically
                                }
                            }
                        
                            // If no note is selected, try to select the first root note
                            if notes_state.get_selected_note().is_none() {
                                // Get the first root note ID first
                                let first_root_id = notes_state.get_root_notes()
                                    .first()
                                    .map(|note| note.id.clone());
                                
                                // Then select it if we found one
                                if let Some(id) = first_root_id {
                                    notes_state.select_note(&id)?;
                                }
                            }
                        }
                    },
                    _ => {}
                }
            }
            KeyCode::F(1) => {
                self.show_help = !self.show_help;
            }
            KeyCode::F(2) => {
                self.show_settings = !self.show_settings;
            }
            KeyCode::Esc => {
                // Close any open dialogs
                if self.show_help {
                    self.show_help = false;
                } else if self.show_settings {
                    self.show_settings = false;
                } else if self.show_task_form {
                    self.reset_form();
                    self.show_task_form = false;
                } else if self.editing_note {
                    self.cancel_note_edit();
                } else if self.show_tag_form {
                    self.cancel_tag_form();
                } else if self.note_search_active {
                    self.note_search_active = false;
                    self.note_search_query = String::new();
                }
            }
            
            // Tab-specific shortcuts
            _ => {
                match self.tab_index {
                    0 => self.handle_timer_keys(key)?,
                    1 => {
                        // If in kanban tab and no task is selected yet, try to select one first
                        if self.selected_task.is_none() {
                            self.select_first_available_task()?;
                        }
                        self.handle_kanban_keys(key)?;
                    },
                    2 => self.handle_notes_keys(key)?,
                    _ => {}
                }
            }
        }
        
        Ok(false)
    }
    
    /// Reset form data
    fn reset_form(&mut self) {
        self.task_form_title = String::new();
        self.task_form_description = String::new();
        self.focused_field = FormField::Title;
    }
    
    /// Open the task form
    fn open_task_form(&mut self) {
        self.reset_form();
        self.show_task_form = true;
    }
    
    /// Handle task form key inputs
    fn handle_task_form_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.reset_form();
                self.show_task_form = false;
            }
            KeyCode::Tab | KeyCode::Char('\t') => {
                // Cycle through form fields
                self.focused_field = match self.focused_field {
                    FormField::Title => FormField::Description,
                    FormField::Description => FormField::CancelButton,
                    FormField::CancelButton => FormField::SaveButton,
                    FormField::SaveButton => FormField::Title,
                };
            }
            KeyCode::BackTab => {
                // Cycle through form fields backwards (Shift+Tab)
                self.focused_field = match self.focused_field {
                    FormField::Title => FormField::SaveButton,
                    FormField::Description => FormField::Title,
                    FormField::CancelButton => FormField::Description,
                    FormField::SaveButton => FormField::CancelButton,
                };
            }
            KeyCode::Enter => {
                match self.focused_field {
                    FormField::CancelButton => {
                        self.reset_form();
                        self.show_task_form = false;
                    }
                    FormField::SaveButton => {
                        self.submit_task_form()?;
                    }
                    // For text fields, move to next field
                    FormField::Title => {
                        self.focused_field = FormField::Description;
                    }
                    FormField::Description => {
                        self.focused_field = FormField::SaveButton;
                    }
                }
            }
            // Handle text input
            KeyCode::Char(c) => {
                match self.focused_field {
                    FormField::Title => {
                        self.task_form_title.push(c);
                    }
                    FormField::Description => {
                        self.task_form_description.push(c);
                    }
                    _ => {}
                }
            }
            KeyCode::Backspace => {
                match self.focused_field {
                    FormField::Title => {
                        self.task_form_title.pop();
                    }
                    FormField::Description => {
                        self.task_form_description.pop();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        
        Ok(false)
    }
    
    /// Submit the task form to create a new task
    fn submit_task_form(&mut self) -> Result<()> {
        // Check if title is not empty
        if self.task_form_title.is_empty() {
            // Could show an error message here
            return Ok(());
        }
        
        // Create task
        {
            let mut kanban = lock_mutex(&self.kanban_state)?;
            
            let title = self.task_form_title.clone();
            let description = self.task_form_description.clone();
            
            // Create task in the selected project or default project
            if kanban.get_selected_project().is_some() {
                // Using a temporary string for project_id to avoid borrow issues
                let project_id = kanban.get_selected_project()
                    .ok_or_else(|| anyhow!("No project selected"))?
                    .id.clone();
                kanban.create_task_in_project(&project_id, &title, &description)?;
            } else {
                // No project selected, create a default project first
                kanban.create_default_project()?;
                // Then get the project ID and create task
                let project_id = kanban.get_selected_project()
                    .ok_or_else(|| anyhow!("No project selected"))?
                    .id.clone();
                kanban.create_task_in_project(&project_id, &title, &description)?;
            }
            
            // Save changes
            let _ = kanban.save_to_disk();
        }
        
        // Reset form and close it
        self.reset_form();
        self.show_task_form = false;
        
        // Explicitly set the selection to the first task in the Todo column
        self.selected_task = Some((0, 0));
        
        Ok(())
    }
    
    /// Handle timer tab key inputs
    fn handle_timer_keys(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('s') => {
                // Start/skip timer
                let mut timer = lock_mutex(&self.timer_state)?;
                if timer.is_running {
                    // If timer is running, skip to next period
                    timer.complete_period();
                } else {
                    // If timer is paused, start it
                    timer.start();
                }
            }
            KeyCode::Char('p') => {
                // Pause timer and save current time
                let mut timer = lock_mutex(&self.timer_state)?;
                if timer.is_running {
                    // Calculate actual elapsed time since start
                    let elapsed_seconds = if timer.is_work_period {
                        if let Some(start_time) = timer.start_time {
                            start_time.elapsed().as_secs()
                        } else {
                            0 // Timer wasn't actually running
                        }
                    } else {
                        0 // Don't track break time
                    };
                    
                    // Get task ID before pausing
                    let task_id = timer.current_task_id.clone();
                    
                    // Pause the timer
                    timer.pause();
                    
                    // If it was a work period and we have a task, save the time
                    if elapsed_seconds > 0 {
                        if let Some(task_id) = task_id {
                            drop(timer); // Release timer lock before acquiring kanban lock
                            
                            // Add the elapsed time to the task
                            let mut kanban = lock_mutex(&self.kanban_state)?;
                            let _ = kanban.add_time_to_task(&task_id, elapsed_seconds);
                            let _ = kanban.save_to_disk();
                        }
                    }
                }
            }
            KeyCode::Char('r') => {
                // Reset timer but first save any elapsed time
                let mut timer = lock_mutex(&self.timer_state)?;
                
                // If timer is running and it's a work period, calculate elapsed time
                if timer.is_running && timer.is_work_period {
                    let elapsed_seconds = if let Some(start_time) = timer.start_time {
                        start_time.elapsed().as_secs()
                    } else {
                        0
                    };
                    
                    // Get task ID before resetting
                    let task_id = timer.current_task_id.clone();
                    
                    // If we have elapsed time and a task, save the time
                    if elapsed_seconds > 0 && task_id.is_some() {
                        let task_id = match task_id {
                            Some(id) => id,
                            None => return Ok(()),
                        };
                        drop(timer); // Release timer lock before acquiring kanban lock
                        
                        // Add the elapsed time to the task
                        let mut kanban = lock_mutex(&self.kanban_state)?;
                        let _ = kanban.add_time_to_task(&task_id, elapsed_seconds);
                        let _ = kanban.save_to_disk();
                        
                        // Re-acquire timer lock
                        timer = self.timer_state.lock().unwrap();
                    }
                }
                
                // Reset the timer
                timer.reset();
            }
            KeyCode::Char('t') => {
                // Toggle between work and break but first save any elapsed time
                let mut timer = lock_mutex(&self.timer_state)?;
                
                // If timer is running and it's a work period, calculate elapsed time
                if timer.is_running && timer.is_work_period {
                    let elapsed_seconds = if let Some(start_time) = timer.start_time {
                        start_time.elapsed().as_secs()
                    } else {
                        0
                    };
                    
                    // Get task ID before toggling
                    let task_id = timer.current_task_id.clone();
                    
                    // If we have elapsed time and a task, save the time
                    if elapsed_seconds > 0 && task_id.is_some() {
                        let task_id = match task_id {
                            Some(id) => id,
                            None => return Ok(()),
                        };
                        drop(timer); // Release timer lock before acquiring kanban lock
                        
                        // Add the elapsed time to the task
                        let mut kanban = lock_mutex(&self.kanban_state)?;
                        let _ = kanban.add_time_to_task(&task_id, elapsed_seconds);
                        let _ = kanban.save_to_disk();
                        
                        // Re-acquire timer lock
                        timer = self.timer_state.lock().unwrap();
                    }
                }
                
                // Toggle between work and break
                let currently_work_period = timer.is_work_period;
                timer.set_work_period(!currently_work_period);
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Handle kanban tab key inputs
    fn handle_kanban_keys(&mut self, key: KeyEvent) -> Result<()> {
        // If we don't have a task selected yet, try to select one first
        if self.selected_task.is_none() {
            self.select_first_available_task()?;
        }
        
        match key.code {
            // Task navigation
            KeyCode::Left | KeyCode::Char('h') => {
                self.move_task_selection_left()?;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.move_task_selection_right()?;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_task_selection_up()?;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_task_selection_down()?;
            }
            
            // Task management
            KeyCode::Char('n') => {
                // Create new task
                self.open_task_form();
            }
            KeyCode::Char('t') => {
                // Add time to selected task
                if let Some((col_idx, task_idx)) = self.selected_task {
                    let mut kanban = lock_mutex(&self.kanban_state)?;
                    if let Some(project) = kanban.get_selected_project() {
                        let tasks = kanban.get_project_tasks(&project.id)?;
                        let column_tasks = self.get_tasks_in_column(col_idx, &tasks);
                        
                        if task_idx < column_tasks.len() {
                            // Get task ID
                            let task_id = column_tasks[task_idx].id.clone();
                            
                            // Add 5 minutes (300 seconds) to the task
                            let _ = kanban.add_time_to_task(&task_id, 300);
                        }
                    }
                }
            }
            KeyCode::Char('i') => {
                // Move task to In Progress column
                if self.selected_task.is_some() {
                    self.move_task_to_column(TaskStatus::InProgress)?;
                }
            }
            KeyCode::Char('d') => {
                // Move task to Done column
                if self.selected_task.is_some() {
                    self.move_task_to_column(TaskStatus::Done)?;
                }
            }
            KeyCode::Char(' ') => {
                if let Some((col_idx, task_idx)) = self.selected_task {
                    // Select task for time tracking
                    let mut kanban = lock_mutex(&self.kanban_state)?;
                    if let Some(project) = kanban.get_selected_project() {
                        let tasks = kanban.get_project_tasks(&project.id)?;
                        
                        // Get tasks in the selected column
                        let column_tasks = match col_idx {
                            0 => tasks.iter().filter(|task| task.status == TaskStatus::Todo).collect::<Vec<_>>(),
                            1 => tasks.iter().filter(|task| task.status == TaskStatus::InProgress).collect::<Vec<_>>(),
                            2 => tasks.iter().filter(|task| task.status == TaskStatus::Done).collect::<Vec<_>>(),
                            _ => Vec::new(),
                        };
                        
                        if task_idx < column_tasks.len() {
                            // Get task ID
                            let task_id = column_tasks[task_idx].id.clone();
                            
                            // If task is not in In Progress, move it there
                            if col_idx != 1 {
                                let task_id_copy = task_id.clone();
                                kanban.update_task_status(&task_id_copy, TaskStatus::InProgress)?;
                                let _ = kanban.save_to_disk();
                                
                                // Update UI selection to match new task position
                                self.selected_task = Some((1, 0)); // First task in In Progress column
                            }
                            
                            drop(kanban); // Release lock
                            
                            // Set current task in timer state
                            let mut timer = lock_mutex(&self.timer_state)?;
                            timer.set_current_task(Some(task_id));
                            
                            // If timer is not already running and it's a work period, start it
                            if !timer.is_running && timer.is_work_period {
                                timer.start();
                            }
                        }
                    }
                }
            }
            KeyCode::Char('x') => {
                // Delete selected task
                if let Some((col_idx, task_idx)) = self.selected_task {
                    let mut kanban = lock_mutex(&self.kanban_state)?;
                    if let Some(project) = kanban.get_selected_project() {
                        // Clone the project ID to avoid borrow issues
                        let project_id = project.id.clone();
                        let tasks = kanban.get_project_tasks(&project_id)?;
                        let column_tasks = self.get_tasks_in_column(col_idx, &tasks);
                        
                        if task_idx < column_tasks.len() {
                            // Get task ID
                            let task_id = column_tasks[task_idx].id.clone();
                            
                            // Delete the task
                            kanban.delete_task(&task_id)?;
                            
                            // Save changes
                            let _ = kanban.save_to_disk();
                            
                            // Get updated tasks for selection
                            let tasks = kanban.get_project_tasks(&project_id)?;
                            let remaining_tasks = self.get_tasks_in_column(col_idx, &tasks);
                            let has_remaining = !remaining_tasks.is_empty();
                            let new_idx = if task_idx >= remaining_tasks.len() {
                                remaining_tasks.len().saturating_sub(1)
                            } else {
                                task_idx
                            };
                            
                            // Drop the lock before updating selection
                            drop(kanban);
                            
                            // Update selection
                            if has_remaining {
                                self.selected_task = Some((col_idx, new_idx));
                            } else {
                                self.select_first_available_task()?;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Get tasks in a specific column
    fn get_tasks_in_column<'a>(&self, column_idx: usize, tasks: &'a [crate::state::kanban_state::Task]) -> Vec<&'a crate::state::kanban_state::Task> {
        match column_idx {
            0 => tasks.iter().filter(|task| task.status == TaskStatus::Todo).collect(),
            1 => tasks.iter().filter(|task| task.status == TaskStatus::InProgress).collect(),
            2 => tasks.iter().filter(|task| task.status == TaskStatus::Done).collect(),
            _ => Vec::new(),
        }
    }
    
    /// Move task selection up
    fn move_task_selection_up(&mut self) -> Result<()> {
        if let Some((col_idx, task_idx)) = self.selected_task {
            // If already at the top, do nothing
            if task_idx > 0 {
                self.selected_task = Some((col_idx, task_idx - 1));
            }
        } else {
            // If no task is selected, select the first task in the first column
            let kanban = lock_mutex(&self.kanban_state)?;
            if let Some(project) = kanban.get_selected_project() {
                let tasks = kanban.get_project_tasks(&project.id)?;
                let todo_tasks = tasks.iter()
                    .filter(|task| task.status == TaskStatus::Todo)
                    .count();
                
                if todo_tasks > 0 {
                    self.selected_task = Some((0, 0));
                }
            }
        }
        
        Ok(())
    }
    
    /// Move task selection down
    fn move_task_selection_down(&mut self) -> Result<()> {
        if let Some((col_idx, task_idx)) = self.selected_task {
            let kanban = lock_mutex(&self.kanban_state)?;
            if let Some(project) = kanban.get_selected_project() {
                let tasks = kanban.get_project_tasks(&project.id)?;
                
                // Get tasks in the current column
                let column_tasks = self.get_tasks_in_column(col_idx, &tasks);
                
                // If not at the bottom, move down
                if task_idx < column_tasks.len() - 1 {
                    self.selected_task = Some((col_idx, task_idx + 1));
                }
            }
        } else {
            // If no task is selected, select the first task in the first column
            let kanban = lock_mutex(&self.kanban_state)?;
            if let Some(project) = kanban.get_selected_project() {
                let tasks = kanban.get_project_tasks(&project.id)?;
                let todo_tasks = tasks.iter()
                    .filter(|task| task.status == TaskStatus::Todo)
                    .count();
                
                if todo_tasks > 0 {
                    self.selected_task = Some((0, 0));
                }
            }
        }
        
        Ok(())
    }
    
    /// Move task selection left
    fn move_task_selection_left(&mut self) -> Result<()> {
        if let Some((col_idx, _)) = self.selected_task {
            // If already at the leftmost column, do nothing
            if col_idx > 0 {
                let kanban = lock_mutex(&self.kanban_state)?;
                if let Some(project) = kanban.get_selected_project() {
                    let tasks = kanban.get_project_tasks(&project.id)?;
                    
                    // Get tasks in the left column
                    let left_col_idx = col_idx - 1;
                    let left_column_tasks = self.get_tasks_in_column(left_col_idx, &tasks);
                    
                    // If there are tasks in the left column, select the first one
                    if !left_column_tasks.is_empty() {
                        self.selected_task = Some((left_col_idx, 0));
                    }
                }
            }
        } else {
            // If no task is selected, select the first task in the first column
            let kanban = lock_mutex(&self.kanban_state)?;
            if let Some(project) = kanban.get_selected_project() {
                let tasks = kanban.get_project_tasks(&project.id)?;
                let todo_tasks = tasks.iter()
                    .filter(|task| task.status == TaskStatus::Todo)
                    .count();
                
                if todo_tasks > 0 {
                    self.selected_task = Some((0, 0));
                }
            }
        }
        
        Ok(())
    }
    
    /// Move task selection right
    fn move_task_selection_right(&mut self) -> Result<()> {
        if let Some((col_idx, _)) = self.selected_task {
            // If already at the rightmost column, do nothing
            if col_idx < 2 {
                let kanban = lock_mutex(&self.kanban_state)?;
                if let Some(project) = kanban.get_selected_project() {
                    let tasks = kanban.get_project_tasks(&project.id)?;
                    
                    // Get tasks in the right column
                    let right_col_idx = col_idx + 1;
                    let right_column_tasks = self.get_tasks_in_column(right_col_idx, &tasks);
                    
                    // If there are tasks in the right column, select the first one
                    if !right_column_tasks.is_empty() {
                        self.selected_task = Some((right_col_idx, 0));
                    }
                }
            }
        } else {
            // If no task is selected, select the first task in the first column
            let kanban = lock_mutex(&self.kanban_state)?;
            if let Some(project) = kanban.get_selected_project() {
                let tasks = kanban.get_project_tasks(&project.id)?;
                let todo_tasks = tasks.iter()
                    .filter(|task| task.status == TaskStatus::Todo)
                    .count();
                
                if todo_tasks > 0 {
                    self.selected_task = Some((0, 0));
                }
            }
        }
        
        Ok(())
    }
    
    /// Move the selected task to a specific column
    pub fn move_task_to_column(&mut self, target_status: TaskStatus) -> Result<()> {
        if let Some((col_idx, task_idx)) = self.selected_task {
            let kanban = lock_mutex(&self.kanban_state)?;
            if let Some(project) = kanban.get_selected_project() {
                let tasks = kanban.get_project_tasks(&project.id)?;
                let column_tasks = self.get_tasks_in_column(col_idx, &tasks);
                
                if task_idx < column_tasks.len() {
                    // Get task ID
                    let task_id = column_tasks[task_idx].id.clone();
                    drop(kanban); // Release lock before updating status
                    
                    // Update the task's status
                    self.kanban_state.lock().unwrap().update_task_status(&task_id, target_status)?;
                    
                    // Verify the task was moved correctly
                    let kanban = lock_mutex(&self.kanban_state)?;
                    let updated_task = kanban.get_task(&task_id)
                        .ok_or_else(|| anyhow!("Task not found after update"))?;
                    
                    assert_eq!(updated_task.status, target_status, "Task status was not updated correctly");
                    drop(kanban);
                    
                    // Save changes
                    self.kanban_state.lock().unwrap().save_to_disk()?;
                    
                    // Update selection to the new column
                    let new_col_idx = match target_status {
                        TaskStatus::Todo => 0,
                        TaskStatus::InProgress => 1,
                        TaskStatus::Done => 2,
                    };
                    self.selected_task = Some((new_col_idx, 0));
                }
            }
        }
        Ok(())
    }
    
    /// Helper method to select the first available task
    pub fn select_first_available_task(&mut self) -> Result<()> {
        let kanban = lock_mutex(&self.kanban_state)?;
        if let Some(project) = kanban.get_selected_project() {
            let tasks = kanban.get_project_tasks(&project.id)?;
            
            // Try to select a task in each column in order: Todo, In Progress, Done
            let todo_tasks = tasks.iter()
                .filter(|task| task.status == TaskStatus::Todo)
                .count();
            
            if todo_tasks > 0 {
                self.selected_task = Some((0, 0)); // First task in Todo column
                return Ok(());
            }
            
            let in_progress_tasks = tasks.iter()
                .filter(|task| task.status == TaskStatus::InProgress)
                .count();
            
            if in_progress_tasks > 0 {
                self.selected_task = Some((1, 0)); // First task in In Progress column
                return Ok(());
            }
            
            let done_tasks = tasks.iter()
                .filter(|task| task.status == TaskStatus::Done)
                .count();
            
            if done_tasks > 0 {
                self.selected_task = Some((2, 0)); // First task in Done column
                return Ok(());
            }
        }
        
        Ok(())
    }
    
    /// Update on tick
    pub fn tick(&mut self) {
        let mut timer = match lock_mutex(&self.timer_state) {
            Ok(timer) => timer,
            Err(_) => return, // Skip tick if mutex is poisoned
        };
        
        // Check if timer running and needs updating
        if timer.is_running {
            timer.tick();
            
            // Check if timer completed
            if timer.is_complete() {
                // Before completing period, calculate the actual elapsed time
                let elapsed_seconds = if timer.is_work_period {
                    if let Some(start_time) = timer.start_time {
                        start_time.elapsed().as_secs()
                    } else {
                        0 // Timer wasn't actually running
                    }
                } else {
                    0 // Don't add break time to task time
                };
                
                // Get task ID before completing period
                let task_id = timer.current_task_id.clone();
                
                // Play notification sound or show notification
                timer.complete_period();
                
                // If it was a work period, record the actual elapsed time
                if let Some(task_id) = task_id {
                    if elapsed_seconds > 0 {
                        drop(timer); // Release timer lock before acquiring kanban lock
                        
                        // Add the actual elapsed time to the task
                        let mut kanban = match lock_mutex(&self.kanban_state) {
                            Ok(kanban) => kanban,
                            Err(_) => return, // Skip if mutex is poisoned
                        };
                        let _ = kanban.add_time_to_task(&task_id, elapsed_seconds);
                        let _ = kanban.save_to_disk();
                    }
                }
            }
        }
    }
}

// Helper function to add visible child notes to the list
fn add_visible_child_notes(notes_state: &NotesState, visible_notes: &mut Vec<String>, parent_id: &str) {
    if let Some(parent) = notes_state.get_note(parent_id) {
        for child_id in &parent.children {
            visible_notes.push(child_id.clone());
            if let Some(child) = notes_state.get_note(child_id) {
                if child.expanded {
                    add_visible_child_notes(notes_state, visible_notes, child_id);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_test_app() -> App {
        App::new().unwrap()
    }

    fn create_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        }
    }

    #[test]
    fn test_tab_navigation() {
        let mut app = create_test_app();
        
        // Test forward tab navigation
        app.handle_key(create_key_event(KeyCode::Tab)).unwrap();
        assert_eq!(app.tab_index, 1);
        
        app.handle_key(create_key_event(KeyCode::Tab)).unwrap();
        assert_eq!(app.tab_index, 2);
        
        app.handle_key(create_key_event(KeyCode::Tab)).unwrap();
        assert_eq!(app.tab_index, 0);
        
        // Test backward tab navigation
        let mut app = create_test_app();
        app.handle_key(KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::SHIFT,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        }).unwrap();
        assert_eq!(app.tab_index, 2);
    }

    #[test]
    fn test_task_creation() {
        let mut app = create_test_app();
        
        // Switch to Kanban tab
        app.handle_key(create_key_event(KeyCode::Tab)).unwrap();
        assert_eq!(app.tab_index, 1);
        
        // Open task form
        app.handle_key(create_key_event(KeyCode::Char('n'))).unwrap();
        assert!(app.show_task_form);
        
        // Enter task title
        app.task_form_title = "Test Task".to_string();
        app.task_form_description = "Test Description".to_string();
        
        // Submit task form
        app.focused_field = FormField::SaveButton;
        app.handle_key(create_key_event(KeyCode::Enter)).unwrap();
        
        // Verify task was created
        let kanban = app.kanban_state.lock().unwrap();
        let project = kanban.get_selected_project().unwrap();
        let tasks = kanban.get_project_tasks(&project.id).unwrap();
        let todo_tasks: Vec<_> = tasks.iter()
            .filter(|task| task.status == TaskStatus::Todo)
            .collect();
        
        assert!(!todo_tasks.is_empty());
        assert_eq!(todo_tasks[0].title, "Test Task");
        assert_eq!(todo_tasks[0].description, "Test Description");
    }

    #[test]
    fn test_timer_controls() {
        let mut app = create_test_app();
        
        // Start timer
        app.handle_key(create_key_event(KeyCode::Char('s'))).unwrap();
        {
            let timer = app.timer_state.lock().unwrap();
            assert!(timer.is_running);
        }
        
        // Pause timer
        app.handle_key(create_key_event(KeyCode::Char('p'))).unwrap();
        {
            let timer = app.timer_state.lock().unwrap();
            assert!(!timer.is_running);
        }
        
        // Reset timer
        app.handle_key(create_key_event(KeyCode::Char('r'))).unwrap();
        {
            let timer = app.timer_state.lock().unwrap();
            assert!(!timer.is_running);
            assert!(timer.get_remaining_seconds() == timer.work_duration.as_secs());
        }
    }

    #[test]
    fn test_notes_creation() {
        let mut app = create_test_app();
        
        // Switch to Notes tab
        app.handle_key(create_key_event(KeyCode::Tab)).unwrap();
        app.handle_key(create_key_event(KeyCode::Tab)).unwrap();
        assert_eq!(app.tab_index, 2);
        
        // Create new note
        app.handle_key(create_key_event(KeyCode::Char('n'))).unwrap();
        assert!(app.editing_note);
        
        // Enter note content
        app.note_form_title = "Test Note".to_string();
        app.note_form_content = "Test Content".to_string();
        
        // Submit note form
        app.focused_note_field = NoteField::SaveButton;
        app.handle_key(create_key_event(KeyCode::Enter)).unwrap();
        
        // Verify note was created
        let notes = app.notes_state.lock().unwrap();
        let root_notes = notes.get_root_notes();
        assert!(!root_notes.is_empty());
        assert_eq!(root_notes[0].title, "Test Note");
        assert_eq!(root_notes[0].content, "Test Content");
    }

    #[test]
    fn test_task_movement() {
        let mut app = create_test_app();
        
        // First, let's check the initial state
        {
            let kanban = app.kanban_state.lock().unwrap();
            let project = kanban.get_selected_project().unwrap();
            let tasks = kanban.get_project_tasks(&project.id).unwrap();
            println!("Initial state:");
            println!("Todo tasks: {:?}", tasks.iter().filter(|t| t.status == TaskStatus::Todo).map(|t| &t.title).collect::<Vec<_>>());
            println!("In Progress tasks: {:?}", tasks.iter().filter(|t| t.status == TaskStatus::InProgress).map(|t| &t.title).collect::<Vec<_>>());
            println!("Done tasks: {:?}", tasks.iter().filter(|t| t.status == TaskStatus::Done).map(|t| &t.title).collect::<Vec<_>>());
        }
        
        // Create a task first
        app.handle_key(create_key_event(KeyCode::Tab)).unwrap();
        app.handle_key(create_key_event(KeyCode::Char('n'))).unwrap();
        app.task_form_title = "Movable Task".to_string();
        app.task_form_description = "Test Description".to_string();
        app.focused_field = FormField::SaveButton;
        app.handle_key(create_key_event(KeyCode::Enter)).unwrap();
        
        // Verify task was created in Todo and check all columns
        {
            let kanban = app.kanban_state.lock().unwrap();
            let project = kanban.get_selected_project().unwrap();
            let tasks = kanban.get_project_tasks(&project.id).unwrap();
            
            let todo_tasks: Vec<_> = tasks.iter()
                .filter(|task| task.status == TaskStatus::Todo)
                .collect();
            let in_progress_tasks: Vec<_> = tasks.iter()
                .filter(|task| task.status == TaskStatus::InProgress)
                .collect();
            let done_tasks: Vec<_> = tasks.iter()
                .filter(|task| task.status == TaskStatus::Done)
                .collect();
            
            println!("\nAfter task creation:");
            println!("Todo tasks: {:?}", todo_tasks.iter().map(|t| &t.title).collect::<Vec<_>>());
            println!("In Progress tasks: {:?}", in_progress_tasks.iter().map(|t| &t.title).collect::<Vec<_>>());
            println!("Done tasks: {:?}", done_tasks.iter().map(|t| &t.title).collect::<Vec<_>>());
            
            assert_eq!(todo_tasks.len(), 1, "Should have exactly one task in Todo");
            assert_eq!(in_progress_tasks.len(), 0, "Should have no tasks in In Progress");
            assert_eq!(done_tasks.len(), 0, "Should have no tasks in Done");
            assert_eq!(todo_tasks[0].title, "Movable Task", "Task title doesn't match");
        }
        
        // Move task to In Progress
        app.handle_key(create_key_event(KeyCode::Char('i'))).unwrap();
        
        // Verify task was moved to In Progress and check all columns
        {
            let kanban = app.kanban_state.lock().unwrap();
            let project = kanban.get_selected_project().unwrap();
            let tasks = kanban.get_project_tasks(&project.id).unwrap();
            
            let todo_tasks: Vec<_> = tasks.iter()
                .filter(|task| task.status == TaskStatus::Todo)
                .collect();
            let in_progress_tasks: Vec<_> = tasks.iter()
                .filter(|task| task.status == TaskStatus::InProgress)
                .collect();
            let done_tasks: Vec<_> = tasks.iter()
                .filter(|task| task.status == TaskStatus::Done)
                .collect();
            
            println!("\nAfter moving to In Progress:");
            println!("Todo tasks: {:?}", todo_tasks.iter().map(|t| &t.title).collect::<Vec<_>>());
            println!("In Progress tasks: {:?}", in_progress_tasks.iter().map(|t| &t.title).collect::<Vec<_>>());
            println!("Done tasks: {:?}", done_tasks.iter().map(|t| &t.title).collect::<Vec<_>>());
            
            assert_eq!(todo_tasks.len(), 0, "Should have no tasks in Todo");
            assert_eq!(in_progress_tasks.len(), 1, "Should have exactly one task in In Progress");
            assert_eq!(done_tasks.len(), 0, "Should have no tasks in Done");
            assert_eq!(in_progress_tasks[0].title, "Movable Task", "Task title doesn't match");
        }
        
        // Move task to Done
        app.handle_key(create_key_event(KeyCode::Char('d'))).unwrap();
        
        // Verify task was moved to Done and check all columns
        {
            let kanban = app.kanban_state.lock().unwrap();
            let project = kanban.get_selected_project().unwrap();
            let tasks = kanban.get_project_tasks(&project.id).unwrap();
            
            let todo_tasks: Vec<_> = tasks.iter()
                .filter(|task| task.status == TaskStatus::Todo)
                .collect();
            let in_progress_tasks: Vec<_> = tasks.iter()
                .filter(|task| task.status == TaskStatus::InProgress)
                .collect();
            let done_tasks: Vec<_> = tasks.iter()
                .filter(|task| task.status == TaskStatus::Done)
                .collect();
            
            println!("\nAfter moving to Done:");
            println!("Todo tasks: {:?}", todo_tasks.iter().map(|t| &t.title).collect::<Vec<_>>());
            println!("In Progress tasks: {:?}", in_progress_tasks.iter().map(|t| &t.title).collect::<Vec<_>>());
            println!("Done tasks: {:?}", done_tasks.iter().map(|t| &t.title).collect::<Vec<_>>());
            
            assert_eq!(todo_tasks.len(), 0, "Should have no tasks in Todo");
            assert_eq!(in_progress_tasks.len(), 0, "Should have no tasks in In Progress");
            assert_eq!(done_tasks.len(), 1, "Should have exactly one task in Done");
            assert_eq!(done_tasks[0].title, "Movable Task", "Task title doesn't match");
        }
    }
}