use anyhow::Result;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

use crate::state::{
    kanban_state::KanbanState,
    timer_state::TimerState,
    notes_state::NotesState,
    save_request::SaveRequest,
};
use crate::utils::mutex::lock_mutex;

// Re-export submodules
pub mod timer;
pub mod kanban;
pub mod notes;
pub mod navigation;

#[cfg(test)]
mod tests;

// The submodules are already public, so we don't need to re-export their contents

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

/// Application mode (which view is active)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Editing,
    Search,
    TagManagement,
}

/// Where to create a new note
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteLocation {
    Root,
    AsChild,
}

/// Application state and logic
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
    /// Whether task detail view is shown
    pub show_task_detail: bool,
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
    /// Create a new application instance
    pub fn new() -> Result<Self> {
        info!("Creating new App instance");
        
        // Initialize app state
        debug!("Initializing app state components");
        let timer_state = Arc::new(Mutex::new(TimerState::default()));
        let kanban_state = Arc::new(Mutex::new(KanbanState::default()));
        let notes_state = Arc::new(Mutex::new(NotesState::default()));
        
        // Load saved data
        {
            debug!("Loading kanban data from disk");
            let mut kanban = lock_mutex(&kanban_state)?;
            if let Err(e) = kanban.load_from_disk() {
                warn!("Failed to load kanban data: {}", e);
                eprintln!("Warning: Failed to load kanban data: {}", e);
            } else {
                info!("Successfully loaded kanban data");
            }
        }
        
        // Load notes data
        {
            debug!("Loading notes data from disk");
            let mut notes = lock_mutex(&notes_state)?;
            if let Err(e) = notes.load_from_disk() {
                warn!("Failed to load notes data: {}", e);
                eprintln!("Warning: Failed to load notes data: {}", e);
            } else {
                info!("Successfully loaded notes data");
            }
            
            // We no longer create a welcome note by default
        }
        
        // Load timer data
        {
            debug!("Loading timer data from disk");
            let mut timer = lock_mutex(&timer_state)?;
            if let Err(e) = timer.load_from_disk() {
                warn!("Failed to load timer data: {}", e);
                eprintln!("Warning: Failed to load timer data: {}", e);
            } else {
                info!("Successfully loaded timer data");
            }
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
            show_task_detail: false,
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
    
    /// Update timer and check for completion
    pub fn tick(&mut self) {
        let (timer_save_request, timer_completed, task_info) = {
            let mut timer = match lock_mutex(&self.timer_state) {
                Ok(timer) => timer,
                Err(e) => {
                    warn!("Failed to lock timer state in tick(): {}", e);
                    return; // Skip tick if mutex is poisoned
                }
            };
            
            if timer.is_running {
                let save_req = timer.tick();
                
                // Check if timer completed
                let remaining = timer.get_remaining_seconds();
                debug!("Timer tick - remaining seconds: {}", remaining);
                
                if remaining == 0 {
                    info!("Timer completed! is_work_period: {}, current_task_id: {:?}", 
                         timer.is_work_period, timer.current_task_id);
                    
                    // Record time if this was a work period with a task
                    let was_work_period = timer.is_work_period;
                    let task_id = timer.current_task_id.clone();
                    let elapsed_seconds = if was_work_period {
                        timer.work_duration.as_secs()
                    } else {
                        timer.break_duration.as_secs()
                    };
                    
                    info!("Timer completion - was_work_period: {}, elapsed_seconds: {}, task_id: {:?}", 
                         was_work_period, elapsed_seconds, task_id);
                    
                    // Play notification sound or show notification
                    let complete_save_req = timer.complete_period();
                    
                    // Merge save requests - if either needs saving, save
                    let final_save_req = if save_req.is_needed() || complete_save_req.is_needed() {
                        SaveRequest::Full
                    } else {
                        SaveRequest::None
                    };
                    
                    // Only record time if it was a work period with a task
                    let task_info = if was_work_period {
                        task_id.map(|id| (id, elapsed_seconds))
                    } else {
                        None
                    };
                    
                    (final_save_req, true, task_info)
                } else {
                    (save_req, false, None)
                }
            } else {
                (SaveRequest::None, false, None)
            }
        }; // Timer lock is dropped here
        
        // Process timer save request outside of mutex lock
        if timer_save_request.is_needed() {
            debug!("Processing timer save request");
            if let Ok(timer) = lock_mutex(&self.timer_state) {
                match timer.process_save_request(&timer_save_request) {
                    Ok(_) => debug!("Timer state saved successfully"),
                    Err(e) => warn!("Failed to save timer state: {}", e),
                }
            }
        }
        
        // If timer completed and it was a work period, record the actual elapsed time
        if timer_completed {
            info!("Timer completed flag is true, checking for task time update");
            if let Some((task_id, elapsed_seconds)) = task_info {
                info!("Adding {} seconds to task {}", elapsed_seconds, task_id);
                if elapsed_seconds > 0 {
                    // Add the actual elapsed time to the task
                    let save_request = {
                        let mut kanban = match lock_mutex(&self.kanban_state) {
                            Ok(kanban) => kanban,
                            Err(e) => {
                                warn!("Failed to lock kanban state: {}", e);
                                return; // Skip if mutex is poisoned
                            }
                        };
                        match kanban.add_time_to_task(&task_id, elapsed_seconds) {
                            Ok((_task, save_req)) => {
                                info!("Successfully added {} seconds to task {}", elapsed_seconds, task_id);
                                Some(save_req)
                            },
                            Err(e) => {
                                warn!("Failed to add time to task {}: {}", task_id, e);
                                None
                            }
                        }
                    }; // Lock is dropped here
                    
                    // Process save request outside of mutex lock
                    if let Some(save_req) = save_request {
                        info!("Processing kanban save request after time update");
                        if let Ok(kanban) = lock_mutex(&self.kanban_state) {
                            match kanban.process_save_request(&save_req) {
                                Ok(_) => info!("Kanban state saved successfully after time update"),
                                Err(e) => warn!("Failed to save kanban state after time update: {}", e),
                            }
                        }
                    }
                    
                    // Now clear the task ID from the timer if we're entering a break period
                    {
                        let mut timer = match lock_mutex(&self.timer_state) {
                            Ok(timer) => timer,
                            Err(e) => {
                                warn!("Failed to lock timer state to clear task: {}", e);
                                return;
                            }
                        };
                        if !timer.is_work_period && timer.current_task_id.is_some() {
                            info!("Clearing task ID from timer as we're now in break period");
                            let save_req = timer.set_current_task(None);
                            if save_req.is_needed() {
                                drop(timer); // Release lock before saving
                                if let Ok(timer) = lock_mutex(&self.timer_state) {
                                    let _ = timer.process_save_request(&save_req);
                                }
                            }
                        }
                    }
                } else {
                    warn!("Elapsed seconds is 0, not updating task time");
                }
            } else {
                info!("No task associated with completed timer");
            }
        }
    }
}