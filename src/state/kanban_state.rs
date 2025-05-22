use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Status of a task in the kanban board
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    /// Task is in the Todo column
    Todo,
    /// Task is in the In Progress column
    InProgress,
    /// Task is in the Done column
    Done,
}

/// Task model for the kanban board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique ID of the task
    pub id: String,
    /// Title of the task
    pub title: String,
    /// Description of the task
    pub description: String,
    /// Current status of the task
    pub status: TaskStatus,
    /// Time spent on the task in seconds
    pub time_spent: u64,
    /// Timestamp when the task was created
    pub created_at: u64,
    /// Timestamp when the task was last updated
    pub updated_at: u64,
}

/// Project model to group tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Unique ID of the project
    pub id: String,
    /// Name of the project
    pub name: String,
    /// IDs of tasks in the project
    pub tasks: Vec<String>,
    /// Timestamp when the project was created
    pub created_at: u64,
    /// Timestamp when the project was last updated
    pub updated_at: u64,
}

/// Kanban state data for serialization
#[derive(Serialize, Deserialize)]
struct KanbanStateData {
    /// Map of project IDs to projects
    projects: HashMap<String, Project>,
    /// Map of task IDs to tasks
    tasks: HashMap<String, Task>,
    /// Next project ID to assign
    next_project_id: u32,
    /// Next task ID to assign
    next_task_id: u32,
    /// ID of the selected project
    selected_project_id: Option<String>,
}

/// Kanban board state
pub struct KanbanState {
    /// Map of project IDs to projects
    pub projects: HashMap<String, Project>,
    /// Map of task IDs to tasks
    pub tasks: HashMap<String, Task>,
    /// Next project ID to assign
    pub next_project_id: u32,
    /// Next task ID to assign
    pub next_task_id: u32,
    /// Path to the data file
    pub data_file_path: PathBuf,
    /// ID of the selected project
    pub selected_project_id: Option<String>,
}

impl Default for KanbanState {
    fn default() -> Self {
        // Get application data directory
        let app_data_dir = home::home_dir()
            .expect("Failed to get home directory")
            .join(".timber-task");
        let data_file_path = app_data_dir.join("kanban_data.json");
        
        Self {
            projects: HashMap::new(),
            tasks: HashMap::new(),
            next_project_id: 1,
            next_task_id: 1,
            data_file_path,
            selected_project_id: None,
        }
    }
}

impl KanbanState {
    /// Save the kanban state to disk
    pub fn save_to_disk(&self) -> Result<()> {
        let data = KanbanStateData {
            projects: self.projects.clone(),
            tasks: self.tasks.clone(),
            next_project_id: self.next_project_id,
            next_task_id: self.next_task_id,
            selected_project_id: self.selected_project_id.clone(),
        };
        
        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| anyhow!("Failed to serialize kanban state: {}", e))?;
        
        fs::create_dir_all(self.data_file_path.parent().unwrap())
            .map_err(|e| anyhow!("Failed to create data directory: {}", e))?;
            
        fs::write(&self.data_file_path, json)
            .map_err(|e| anyhow!("Failed to write kanban state to disk: {}", e))?;
            
        Ok(())
    }
    
    /// Load the kanban state from disk
    pub fn load_from_disk(&mut self) -> Result<()> {
        if !self.data_file_path.exists() {
            // No file yet, start with a default project
            self.create_default_project()?;
            return Ok(());
        }
        
        let json = fs::read_to_string(&self.data_file_path)
            .map_err(|e| anyhow!("Failed to read kanban state from disk: {}", e))?;
            
        let data: KanbanStateData = serde_json::from_str(&json)
            .map_err(|e| anyhow!("Failed to deserialize kanban state: {}", e))?;
            
        self.projects = data.projects;
        self.tasks = data.tasks;
        self.next_project_id = data.next_project_id;
        self.next_task_id = data.next_task_id;
        self.selected_project_id = data.selected_project_id;
        
        Ok(())
    }
    
    /// Create a new project
    pub fn create_project(&mut self, name: &str) -> Result<Project> {
        let id = self.next_project_id.to_string();
        self.next_project_id += 1;
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let project = Project {
            id: id.clone(),
            name: name.to_string(),
            tasks: Vec::new(),
            created_at: timestamp,
            updated_at: timestamp,
        };
        
        self.projects.insert(id.clone(), project.clone());
        
        // If no project is selected, select this one
        if self.selected_project_id.is_none() {
            self.selected_project_id = Some(id.clone());
        }
        
        // Save changes to disk
        self.save_to_disk()?;
        
        Ok(project)
    }
    
    /// Create a default project if none exists
    pub fn create_default_project(&mut self) -> Result<()> {
        self.create_project("Default Project")?;
        Ok(())
    }
    
    /// Get the selected project
    pub fn get_selected_project(&self) -> Option<&Project> {
        self.selected_project_id.as_ref().and_then(|id| self.projects.get(id))
    }
    
    /// Set the selected project
    #[allow(dead_code)]
    pub fn set_selected_project(&mut self, project_id: &str) -> Result<()> {
        if self.projects.contains_key(project_id) {
            self.selected_project_id = Some(project_id.to_string());
            self.save_to_disk()?;
            Ok(())
        } else {
            Err(anyhow!("Project not found"))
        }
    }
    
    /// Create a new task
    #[allow(dead_code)]
    pub fn create_task(&mut self, title: &str, description: &str) -> Result<Task> {
        let project_id = self.selected_project_id.clone()
            .ok_or_else(|| anyhow!("No project selected"))?;
        
        self.create_task_in_project(&project_id, title, description)
    }
    
    /// Create a new task in a specific project
    pub fn create_task_in_project(&mut self, project_id: &str, title: &str, description: &str) -> Result<Task> {
        // Verify project exists
        if !self.projects.contains_key(project_id) {
            return Err(anyhow!("Project not found"));
        }
        
        let id = self.next_task_id.to_string();
        self.next_task_id += 1;
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let task = Task {
            id: id.clone(),
            title: title.to_string(),
            description: description.to_string(),
            status: TaskStatus::Todo,
            time_spent: 0,
            created_at: timestamp,
            updated_at: timestamp,
        };
        
        // Add task to the project
        if let Some(project) = self.projects.get_mut(project_id) {
            project.tasks.push(id.clone());
            project.updated_at = timestamp;
        }
        
        self.tasks.insert(id.clone(), task.clone());
        
        // Save changes to disk
        self.save_to_disk()?;
        
        Ok(task)
    }
    
    /// Get a task by ID
    pub fn get_task(&self, task_id: &str) -> Option<&Task> {
        self.tasks.get(task_id)
    }
    
    /// Get all tasks for a project
    pub fn get_project_tasks(&self, project_id: &str) -> Result<Vec<Task>> {
        let project = self.projects.get(project_id)
            .ok_or_else(|| anyhow!("Project not found"))?;
        
        let tasks = project.tasks.iter()
            .filter_map(|task_id| self.tasks.get(task_id))
            .cloned()
            .collect();
        
        Ok(tasks)
    }
    
    /// Update a task's status
    pub fn update_task_status(&mut self, task_id: &str, status: TaskStatus) -> Result<Task> {
        // First verify the task exists
        let _task = self.tasks.get(task_id)
            .ok_or_else(|| anyhow!("Task not found"))?;
        
        // Get the project that contains this task
        let _project_id = self.projects.iter()
            .find(|(_, project)| project.tasks.contains(&task_id.to_string()))
            .map(|(id, _)| id.clone())
            .ok_or_else(|| anyhow!("Task not found in any project"))?;
        
        // Update the task's status
        {
            let task = self.tasks.get_mut(task_id).unwrap();
            task.status = status;
            task.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        
        // Save changes to disk
        self.save_to_disk()?;
        
        Ok(self.tasks.get(task_id).unwrap().clone())
    }
    
    /// Update task time
    #[allow(dead_code)]
    pub fn update_task_time(&mut self, task_id: &str) -> Result<Task> {
        {
            let task = self.tasks.get_mut(task_id)
                .ok_or_else(|| anyhow!("Task not found"))?;
            
            // Time is tracked automatically in the timer, so we don't need to update it here
            // This function just ensures the task's updated_at timestamp is updated
            task.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        
        // Save changes to disk
        self.save_to_disk()?;
        
        Ok(self.tasks.get(task_id).unwrap().clone())
    }
    
    /// Add time to a task (used when tracking time with the timer)
    pub fn add_time_to_task(&mut self, task_id: &str, seconds: u64) -> Result<Task> {
        {
            let task = self.tasks.get_mut(task_id)
                .ok_or_else(|| anyhow!("Task not found"))?;
            
            task.time_spent += seconds;
            task.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        
        // Save changes to disk
        self.save_to_disk()?;
        
        Ok(self.tasks.get(task_id).unwrap().clone())
    }

    /// Delete a task
    pub fn delete_task(&mut self, task_id: &str) -> Result<()> {
        // Remove task from all projects
        for project in self.projects.values_mut() {
            project.tasks.retain(|id| id != task_id);
        }
        
        // Remove task from tasks map
        self.tasks.remove(task_id)
            .ok_or_else(|| anyhow!("Task not found"))?;
        
        // Save changes to disk
        self.save_to_disk()?;
        
        Ok(())
    }
}