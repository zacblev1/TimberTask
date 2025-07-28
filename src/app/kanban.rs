use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use super::{App, FormField};
use crate::state::kanban_state::{KanbanState, TaskStatus};
use crate::state::save_request::SaveRequest;
use crate::utils::mutex::lock_mutex;

impl App {
    /// Handle kanban board key inputs
    pub fn handle_kanban_keys(&mut self, key: KeyEvent) -> Result<bool> {
        // If we have a selected task, handle task-specific keys
        if let Some((col, row)) = self.selected_task {
            match key.code {
                // Navigation
                KeyCode::Up | KeyCode::Char('k') => {
                    if row > 0 {
                        self.selected_task = Some((col, row - 1));
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let kanban = lock_mutex(&self.kanban_state)?;
                    let max_row = self.get_max_row_for_column(&kanban, col);
                    if row < max_row {
                        self.selected_task = Some((col, row + 1));
                    }
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    if col > 0 {
                        self.move_to_column(col - 1)?;
                    }
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    if col < 2 {
                        self.move_to_column(col + 1)?;
                    }
                }
                
                // Task management
                KeyCode::Char('n') => {
                    self.open_task_form();
                }
                KeyCode::Char('d') => {
                    self.delete_selected_task()?;
                }
                KeyCode::Char('i') => {
                    // Move task to In Progress
                    self.move_task_to_status(TaskStatus::InProgress)?;
                }
                KeyCode::Char('c') => {
                    // Move task to Done (c for complete)
                    self.move_task_to_status(TaskStatus::Done)?;
                }
                KeyCode::Char(' ') => {
                    // Toggle task tracking in timer
                    tracing::debug!("Space key pressed in kanban view");
                    self.toggle_task_tracking()?;
                }
                KeyCode::Char('v') => {
                    // Show task detail view
                    self.show_task_detail = true;
                }
                _ => {}
            }
        } else {
            // No task selected, handle general navigation
            match key.code {
                KeyCode::Char('n') => {
                    self.open_task_form();
                }
                _ => {
                    // Try to select first task
                    self.select_first_available_task()?;
                }
            }
        }
        
        Ok(false)
    }
    
    /// Handle task detail view key inputs
    pub fn handle_task_detail_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('v') => {
                self.show_task_detail = false;
            }
            _ => {}
        }
        Ok(false)
    }
    
    /// Handle task form key inputs
    pub fn handle_task_form_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.show_task_form = false;
                self.task_form_title = String::new();
                self.task_form_description = String::new();
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
                        self.show_task_form = false;
                        self.task_form_title = String::new();
                        self.task_form_description = String::new();
                    }
                    FormField::SaveButton => {
                        if !self.task_form_title.is_empty() {
                            // Create the task and get save request
                            let save_request = {
                                let kanban = self.kanban_state.clone();
                                let mut kanban = lock_mutex(&kanban)?;
                                
                                if let Some(project) = kanban.get_selected_project() {
                                    let project_id = project.id.clone();
                                    match kanban.create_task_in_project(
                                        &project_id,
                                        &self.task_form_title,
                                        &self.task_form_description,
                                    ) {
                                        Ok((_task, save_req)) => Some(save_req),
                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                }
                            }; // Lock is dropped here
                            
                            // Process save request outside of mutex lock
                            if let Some(save_req) = save_request {
                                let kanban = lock_mutex(&self.kanban_state)?;
                                kanban.process_save_request(&save_req)?;
                            }
                            
                            // Close form
                            self.show_task_form = false;
                            self.task_form_title = String::new();
                            self.task_form_description = String::new();
                            
                            // Try to select the new task
                            self.select_first_available_task()?;
                        }
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
    
    /// Get the maximum row index for a given column
    pub fn get_max_row_for_column(&self, kanban: &KanbanState, col: usize) -> usize {
        let project = match kanban.get_selected_project() {
            Some(p) => p,
            None => return 0,
        };
        
        let tasks = kanban.get_project_tasks(&project.id).unwrap_or_default();
        
        let count = match col {
            0 => tasks.iter().filter(|t| t.status == TaskStatus::Todo).count(),
            1 => tasks.iter().filter(|t| t.status == TaskStatus::InProgress).count(),
            2 => tasks.iter().filter(|t| t.status == TaskStatus::Done).count(),
            _ => 0,
        };
        
        if count > 0 { count - 1 } else { 0 }
    }
    
    /// Move selection to a specific column, adjusting row if necessary
    pub fn move_to_column(&mut self, new_col: usize) -> Result<()> {
        let kanban = lock_mutex(&self.kanban_state)?;
        
        if let Some((_, current_row)) = self.selected_task {
            let max_row = self.get_max_row_for_column(&kanban, new_col);
            
            if max_row == 0 && self.count_tasks_in_column(&kanban, new_col) == 0 {
                // No tasks in the target column
                return Ok(());
            }
            
            let new_row = if current_row > max_row {
                max_row
            } else {
                current_row
            };
            
            self.selected_task = Some((new_col, new_row));
        }
        
        Ok(())
    }
    
    /// Count tasks in a specific column
    pub fn count_tasks_in_column(&self, kanban: &KanbanState, col: usize) -> usize {
        let project = match kanban.get_selected_project() {
            Some(p) => p,
            None => return 0,
        };
        
        let tasks = kanban.get_project_tasks(&project.id).unwrap_or_default();
        
        match col {
            0 => tasks.iter().filter(|t| t.status == TaskStatus::Todo).count(),
            1 => tasks.iter().filter(|t| t.status == TaskStatus::InProgress).count(),
            2 => tasks.iter().filter(|t| t.status == TaskStatus::Done).count(),
            _ => 0,
        }
    }
    
    /// Select the first available task on the board
    pub fn select_first_available_task(&mut self) -> Result<()> {
        tracing::debug!("select_first_available_task called");
        let kanban = lock_mutex(&self.kanban_state)?;
        
        // Try each column in order
        for col in 0..3 {
            let count = self.count_tasks_in_column(&kanban, col);
            tracing::debug!("Column {} has {} tasks", col, count);
            if count > 0 {
                self.selected_task = Some((col, 0));
                tracing::debug!("Selected first task in column {}", col);
                return Ok(());
            }
        }
        
        // No tasks found
        tracing::debug!("No tasks found on board");
        self.selected_task = None;
        Ok(())
    }
    
    /// Open the task form
    fn open_task_form(&mut self) {
        self.show_task_form = true;
        self.focused_field = FormField::Title;
        self.task_form_title = String::new();
        self.task_form_description = String::new();
    }
    
    /// Delete the currently selected task
    fn delete_selected_task(&mut self) -> Result<()> {
        if let Some((col, row)) = self.selected_task {
            // Perform deletion and get info needed for selection adjustment
            let (deleted, tasks_remaining) = {
                let kanban = self.kanban_state.clone();
                let mut kanban = lock_mutex(&kanban)?;
                
                if let Some(project) = kanban.get_selected_project() {
                    let project_id = project.id.clone();
                    let tasks = kanban.get_project_tasks(&project_id)?;
                    
                    // Find the task at the selected position
                    let target_status = match col {
                        0 => TaskStatus::Todo,
                        1 => TaskStatus::InProgress,
                        2 => TaskStatus::Done,
                        _ => return Ok(()),
                    };
                    
                    let task_in_column: Vec<_> = tasks
                        .iter()
                        .filter(|t| t.status == target_status)
                        .collect();
                    
                    if let Some(task) = task_in_column.get(row) {
                        let task_id = task.id.clone();
                        let save_request = kanban.delete_task(&task_id)?;
                        
                        // Process save request immediately since we're in a complex operation
                        kanban.process_save_request(&save_request)?;
                        
                        // Return info about remaining tasks
                        let remaining = task_in_column.len() - 1;
                        (true, remaining)
                    } else {
                        (false, task_in_column.len())
                    }
                } else {
                    (false, 0)
                }
            }; // Lock is dropped here
            
            // Adjust selection after deletion
            if deleted {
                let kanban = lock_mutex(&self.kanban_state)?;
                let max_row = self.get_max_row_for_column(&kanban, col);
                
                if tasks_remaining == 0 {
                    // No more tasks in this column, try to find another task
                    drop(kanban);
                    self.select_first_available_task()?;
                } else if row > max_row {
                    // We were on the last task, move up
                    self.selected_task = Some((col, max_row));
                }
                // else: keep the same position (next task moved into this position)
            }
        }
        
        Ok(())
    }
    
    /// Move the selected task to a specific status
    fn move_task_to_status(&mut self, target_status: TaskStatus) -> Result<()> {
        if let Some((col, row)) = self.selected_task {
            // Get task ID and perform the move
            let task_id = {
                let kanban = lock_mutex(&self.kanban_state)?;
                
                if let Some(project) = kanban.get_selected_project() {
                    let project_id = project.id.clone();
                    let tasks = kanban.get_project_tasks(&project_id)?;
                    
                    // Find the task at the selected position
                    let current_status = match col {
                        0 => TaskStatus::Todo,
                        1 => TaskStatus::InProgress,
                        2 => TaskStatus::Done,
                        _ => return Ok(()),
                    };
                    
                    let task_in_column: Vec<_> = tasks
                        .iter()
                        .filter(|t| t.status == current_status)
                        .collect();
                    
                    task_in_column.get(row).map(|t| t.id.clone())
                } else {
                    None
                }
            }; // Lock is dropped here
            
            if let Some(task_id) = task_id {
                // Update the task status and get save request
                let save_request = {
                    let mut kanban = lock_mutex(&self.kanban_state)?;
                    let (_task, save_req) = kanban.update_task_status(&task_id, target_status)?;
                    save_req
                }; // Lock is dropped here
                
                // Process save request outside of mutex lock
                {
                    let kanban = lock_mutex(&self.kanban_state)?;
                    kanban.process_save_request(&save_request)?;
                }
                
                // Find the task's new position and update selection
                let kanban = lock_mutex(&self.kanban_state)?;
                if let Some(project) = kanban.get_selected_project() {
                    let new_col = match target_status {
                        TaskStatus::Todo => 0,
                        TaskStatus::InProgress => 1,
                        TaskStatus::Done => 2,
                    };
                    
                    let tasks = kanban.get_project_tasks(&project.id)?;
                    let tasks_in_new_column: Vec<_> = tasks
                        .iter()
                        .filter(|t| t.status == target_status)
                        .collect();
                    
                    // Find the index of our moved task
                    if let Some(new_row) = tasks_in_new_column.iter()
                        .position(|t| t.id == task_id) {
                        self.selected_task = Some((new_col, new_row));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get the currently selected task ID
    pub fn get_selected_task_id(&self) -> Option<String> {
        tracing::debug!("get_selected_task_id called");
        
        if let Some((col, row)) = self.selected_task {
            tracing::debug!("Getting task at column: {}, row: {}", col, row);
            
            let kanban = match lock_mutex(&self.kanban_state) {
                Ok(k) => k,
                Err(e) => {
                    tracing::error!("Failed to lock kanban state: {}", e);
                    return None;
                }
            };
            
            if let Some(project) = kanban.get_selected_project() {
                tracing::debug!("Selected project: {}", project.id);
                
                let tasks = match kanban.get_project_tasks(&project.id) {
                    Ok(t) => t,
                    Err(e) => {
                        tracing::error!("Failed to get project tasks: {}", e);
                        return None;
                    }
                };
                
                tracing::debug!("Total tasks in project: {}", tasks.len());
                
                let target_status = match col {
                    0 => TaskStatus::Todo,
                    1 => TaskStatus::InProgress,
                    2 => TaskStatus::Done,
                    _ => {
                        tracing::error!("Invalid column: {}", col);
                        return None;
                    }
                };
                
                let task_in_column: Vec<_> = tasks
                    .iter()
                    .filter(|t| t.status == target_status)
                    .collect();
                
                tracing::debug!("Tasks in column {}: {}", col, task_in_column.len());
                
                if let Some(task) = task_in_column.get(row) {
                    tracing::debug!("Found task: id={}, title={}", task.id, task.title);
                    Some(task.id.clone())
                } else {
                    tracing::debug!("No task found at row {}", row);
                    None
                }
            } else {
                tracing::debug!("No project selected");
                None
            }
        } else {
            tracing::debug!("No task selected (selected_task is None)");
            None
        }
    }
    
    /// Toggle tracking for the selected task
    fn toggle_task_tracking(&mut self) -> Result<()> {
        tracing::debug!("toggle_task_tracking called");
        
        // Get the selected task and its current status
        if let Some((col, _row)) = self.selected_task {
            tracing::debug!("Selected task at column: {}", col);
            
            // Handle based on column
            match col {
                0 => {
                    // Task is in TODO column - move to In Progress and start tracking
                    tracing::debug!("Task in TODO column, moving to In Progress and starting tracking");
                    
                    // First move the task to In Progress
                    self.move_task_to_status(TaskStatus::InProgress)?;
                    
                    // Now get the task ID and start tracking
                    if let Some(task_id) = self.get_selected_task_id() {
                        self.start_tracking_task(task_id)?;
                    }
                }
                1 => {
                    // Task is in In Progress column - toggle tracking
                    tracing::debug!("Task in In Progress column, toggling tracking");
                    
                    if let Some(task_id) = self.get_selected_task_id() {
                        let timer = lock_mutex(&self.timer_state)?;
                        let is_tracking_this = timer.current_task_id.as_ref() == Some(&task_id);
                        drop(timer);
                        
                        if is_tracking_this {
                            // Stop tracking
                            self.stop_tracking_task()?;
                        } else {
                            // Start tracking this task
                            self.start_tracking_task(task_id)?;
                        }
                    }
                }
                2 => {
                    // Task is in Done column - show message
                    tracing::debug!("Task in Done column, cannot track completed tasks");
                    self.show_status_message("Cannot track completed tasks");
                }
                _ => {
                    tracing::debug!("Invalid column");
                }
            }
        } else {
            tracing::debug!("No task selected");
        }
        
        Ok(())
    }
    
    /// Start tracking a specific task
    fn start_tracking_task(&mut self, task_id: String) -> Result<()> {
        tracing::info!("Starting to track task: {}", task_id);
        
        let save_request = {
            let mut timer = lock_mutex(&self.timer_state)?;
            let save_req = timer.set_current_task(Some(task_id.clone()));
            
            // If timer isn't running, start it
            if !timer.is_running {
                tracing::info!("Timer not running, starting it");
                let start_req = timer.start();
                // Merge save requests
                if save_req.is_needed() || start_req.is_needed() {
                    SaveRequest::Full
                } else {
                    SaveRequest::None
                }
            } else {
                tracing::info!("Timer already running");
                save_req
            }
        };
        
        // Save timer state outside of lock
        if save_request.is_needed() {
            tracing::info!("Saving timer state after starting task tracking");
            let timer = lock_mutex(&self.timer_state)?;
            timer.process_save_request(&save_request)?;
        }
        
        tracing::info!("Task tracking started successfully");
        Ok(())
    }
    
    /// Stop tracking any task
    fn stop_tracking_task(&mut self) -> Result<()> {
        tracing::debug!("Stopping task tracking");
        
        let save_request = {
            let mut timer = lock_mutex(&self.timer_state)?;
            timer.set_current_task(None)
        };
        
        // Save timer state outside of lock
        if save_request.is_needed() {
            let timer = lock_mutex(&self.timer_state)?;
            timer.process_save_request(&save_request)?;
        }
        
        Ok(())
    }
    
    /// Show a temporary status message to the user
    fn show_status_message(&mut self, _message: &str) {
        // For now, just log it. In the future, we can add a status bar
        // to display temporary messages to the user
        tracing::info!("Status message: {}", _message);
    }
}