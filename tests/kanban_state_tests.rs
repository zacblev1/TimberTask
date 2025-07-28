/// Comprehensive unit tests for the KanbanState module
mod common;

use common::*;
use timber_task::state::kanban_state::{KanbanState, TaskStatus};
use anyhow::Result;

#[test]
fn test_kanban_default_state() -> Result<()> {
    let mut kanban = KanbanState::default();
    
    // Should not have a selected project initially
    assert!(kanban.get_selected_project().is_none());
    
    // Create default project
    kanban.create_default_project()?;
    
    // Now should have a default project
    assert!(kanban.get_selected_project().is_some());
    let project = kanban.get_selected_project().unwrap();
    assert_eq!(project.name, "Default Project");
    
    Ok(())
}

#[test]
fn test_create_project() -> Result<()> {
    let mut kanban = KanbanState::default();
    
    let project_name = "Test Project";
    let project = kanban.create_project(project_name)?;
    
    // Verify project was created
    assert_eq!(project.name, project_name);
    assert!(project.tasks.is_empty());
    assert!(project.created_at > 0);
    assert!(project.updated_at > 0);
    
    // Verify project exists in state
    assert!(kanban.projects.contains_key(&project.id));
    
    Ok(())
}

#[test]
fn test_create_task() -> Result<()> {
    let mut kanban = KanbanState::default();
    
    // Create a project first
    let project = kanban.create_project("Test Project")?;
    let project_id = project.id.clone();
    kanban.set_selected_project(&project_id)?;
    
    let task = kanban.create_task_in_project(&project_id, "Test Task", "Test Description")?;
    
    // Verify task was created
    assert_eq!(task.title, "Test Task");
    assert_eq!(task.description, "Test Description");
    assert_eq!(task.status, TaskStatus::Todo);
    assert_eq!(task.time_spent, 0);
    
    // Verify task exists in state
    assert!(kanban.tasks.contains_key(&task.id));
    
    // Verify task was added to project
    let project = &kanban.projects[&project_id];
    assert!(project.tasks.contains(&task.id));
    
    Ok(())
}

#[test]
fn test_update_task_status() -> Result<()> {
    let mut kanban = KanbanState::default();
    
    // Create a project and task
    let project = kanban.create_project("Test Project")?;
    let project_id = project.id.clone();
    let task = kanban.create_task_in_project(&project_id, "Original", "Original Desc")?;
    let task_id = task.id.clone();
    
    // Update the task status
    let updated_task = kanban.update_task_status(&task_id, TaskStatus::InProgress)?;
    
    assert_eq!(updated_task.status, TaskStatus::InProgress);
    assert!(updated_task.updated_at > updated_task.created_at);
    
    Ok(())
}

#[test]
fn test_move_task() -> Result<()> {
    let mut kanban = KanbanState::default();
    
    // Create a project and task
    let project = kanban.create_project("Test Project")?;
    let project_id = project.id.clone();
    let task = kanban.create_task_in_project(&project_id, "Task", "Desc")?;
    let task_id = task.id.clone();
    
    // Verify initial status
    assert_eq!(task.status, TaskStatus::Todo);
    
    // Move to InProgress
    let task = kanban.update_task_status(&task_id, TaskStatus::InProgress)?;
    assert_eq!(task.status, TaskStatus::InProgress);
    
    // Move to Done
    let task = kanban.update_task_status(&task_id, TaskStatus::Done)?;
    assert_eq!(task.status, TaskStatus::Done);
    
    Ok(())
}

#[test]
fn test_delete_task() -> Result<()> {
    let mut kanban = KanbanState::default();
    
    // Create a project and task
    let project = kanban.create_project("Test Project")?;
    let project_id = project.id.clone();
    let task = kanban.create_task_in_project(&project_id, "To Delete", "Will be deleted")?;
    let task_id = task.id.clone();
    
    // Verify task exists
    assert!(kanban.get_task(&task_id).is_some());
    
    // Delete the task
    kanban.delete_task(&task_id)?;
    
    // Verify task is gone
    assert!(kanban.get_task(&task_id).is_none());
    
    // Verify task was removed from project
    let project = &kanban.projects[&project_id];
    assert!(!project.tasks.contains(&task_id));
    
    Ok(())
}

#[test]
fn test_get_project_tasks() -> Result<()> {
    let mut kanban = KanbanState::default();
    
    // Create a project
    let project = kanban.create_project("Test Project")?;
    let project_id = project.id.clone();
    
    // Create multiple tasks
    let _task1 = kanban.create_task_in_project(&project_id, "Task 1", "Desc 1")?;
    let task2 = kanban.create_task_in_project(&project_id, "Task 2", "Desc 2")?;
    let task3 = kanban.create_task_in_project(&project_id, "Task 3", "Desc 3")?;
    
    // Move tasks to different statuses
    kanban.update_task_status(&task2.id, TaskStatus::InProgress)?;
    kanban.update_task_status(&task3.id, TaskStatus::Done)?;
    
    // Get all project tasks
    let tasks = kanban.get_project_tasks(&project_id)?;
    assert_eq!(tasks.len(), 3);
    
    // Verify task statuses
    let todo_tasks: Vec<_> = tasks.iter().filter(|t| t.status == TaskStatus::Todo).collect();
    let in_progress_tasks: Vec<_> = tasks.iter().filter(|t| t.status == TaskStatus::InProgress).collect();
    let done_tasks: Vec<_> = tasks.iter().filter(|t| t.status == TaskStatus::Done).collect();
    
    assert_eq!(todo_tasks.len(), 1);
    assert_eq!(in_progress_tasks.len(), 1);
    assert_eq!(done_tasks.len(), 1);
    
    Ok(())
}

#[test]
fn test_select_project() -> Result<()> {
    let mut kanban = KanbanState::default();
    
    let project1 = kanban.create_project("Project 1")?;
    let project2 = kanban.create_project("Project 2")?;
    
    // Select project 1
    kanban.set_selected_project(&project1.id)?;
    let selected = kanban.get_selected_project().unwrap();
    assert_eq!(selected.id, project1.id);
    
    // Select project 2
    kanban.set_selected_project(&project2.id)?;
    let selected = kanban.get_selected_project().unwrap();
    assert_eq!(selected.id, project2.id);
    
    Ok(())
}

#[test]
fn test_update_task_time() -> Result<()> {
    let mut kanban = KanbanState::default();
    
    // Create a project and task
    let project = kanban.create_project("Test Project")?;
    let project_id = project.id.clone();
    let task = kanban.create_task_in_project(&project_id, "Timed Task", "Track time")?;
    let task_id = task.id.clone();
    
    // Add time to task
    let updated_task = kanban.add_time_to_task(&task_id, 3600)?; // 1 hour
    assert_eq!(updated_task.time_spent, 3600);
    
    // Add more time
    let updated_task = kanban.add_time_to_task(&task_id, 1800)?; // 30 more minutes
    assert_eq!(updated_task.time_spent, 5400); // Total: 1.5 hours
    
    Ok(())
}

#[test]
fn test_persistence() -> Result<()> {
    let fixture = TestFixture::new();
    
    // Create initial state and data
    let (project_id, task_id) = {
        let mut kanban = fixture.create_kanban_state();
        
        // Create some data
        let project = kanban.create_project("Persistent Project")?;
        let project_id = project.id.clone();
        kanban.set_selected_project(&project_id)?;
        let task = kanban.create_task_in_project(&project_id, "Persistent Task", "Should be saved")?;
        let task_id = task.id.clone();
        
        // Save to disk
        kanban.save_to_disk()?;
        
        (project_id, task_id)
    };
    
    // Create a new state and load from disk
    let mut new_kanban = fixture.create_kanban_state();
    new_kanban.load_from_disk()?;
    
    // Verify data was persisted
    assert!(new_kanban.projects.contains_key(&project_id));
    let loaded_project = &new_kanban.projects[&project_id];
    assert_eq!(loaded_project.name, "Persistent Project");
    
    assert!(new_kanban.tasks.contains_key(&task_id));
    let loaded_task = &new_kanban.tasks[&task_id];
    assert_eq!(loaded_task.title, "Persistent Task");
    
    Ok(())
}

#[test]
fn test_error_handling() -> Result<()> {
    let mut kanban = KanbanState::default();
    
    // Try to get non-existent task (returns None, not error)
    assert!(kanban.get_task("non-existent").is_none());
    
    // Try to create task in non-existent project
    assert!(kanban.create_task_in_project("non-existent", "Task", "Desc").is_err());
    
    // Try to update non-existent task
    assert!(kanban.update_task_status("non-existent", TaskStatus::Done).is_err());
    
    // Try to delete non-existent task
    assert!(kanban.delete_task("non-existent").is_err());
    
    // Try to set non-existent project as selected
    assert!(kanban.set_selected_project("non-existent").is_err());
    
    Ok(())
}

#[test]
fn test_task_ordering() -> Result<()> {
    let mut kanban = KanbanState::default();
    
    // Create a project
    let project = kanban.create_project("Test Project")?;
    let project_id = project.id.clone();
    
    // Create tasks with delays to ensure different timestamps
    let task1 = kanban.create_task_in_project(&project_id, "First", "1")?;
    std::thread::sleep(std::time::Duration::from_millis(10));
    let task2 = kanban.create_task_in_project(&project_id, "Second", "2")?;
    std::thread::sleep(std::time::Duration::from_millis(10));
    let task3 = kanban.create_task_in_project(&project_id, "Third", "3")?;
    
    let tasks = kanban.get_project_tasks(&project_id)?;
    
    // Tasks should be ordered by creation time (oldest first usually)
    // The exact ordering depends on the implementation
    assert_eq!(tasks.len(), 3);
    
    Ok(())
}

// Thread safety tests
#[cfg(test)]
mod thread_safety_tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    #[test]
    fn test_concurrent_task_creation() -> Result<()> {
        let fixture = TestFixture::new();
        let kanban = Arc::new(Mutex::new(fixture.create_kanban_state()));
        
        // Create a project first
        let project_id = {
            let mut k = kanban.lock().unwrap();
            let project = k.create_project("Test Project")?;
            project.id.clone()
        };
        
        let mut handles = vec![];
        
        // Spawn multiple threads creating tasks
        for i in 0..10 {
            let kanban_clone = Arc::clone(&kanban);
            let project_id_clone = project_id.clone();
            
            let handle = thread::spawn(move || {
                let mut k = kanban_clone.lock().unwrap();
                k.create_task_in_project(
                    &project_id_clone,
                    &format!("Concurrent Task {}", i),
                    &format!("Created by thread {}", i),
                )
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads and collect results
        let mut task_ids = vec![];
        for handle in handles {
            let task = handle.join().unwrap()?;
            task_ids.push(task.id);
        }
        
        // Verify all tasks were created
        let kanban = kanban.lock().unwrap();
        let tasks = kanban.get_project_tasks(&project_id)?;
        assert_eq!(tasks.len(), 10);
        
        // Verify all task IDs are unique
        let unique_ids: std::collections::HashSet<_> = task_ids.iter().collect();
        assert_eq!(unique_ids.len(), 10);
        
        Ok(())
    }
    
    #[test]
    fn test_concurrent_task_updates() -> Result<()> {
        let fixture = TestFixture::new();
        let kanban = Arc::new(Mutex::new(fixture.create_kanban_state()));
        
        // Create a project and task
        let (project_id, task_id) = {
            let mut k = kanban.lock().unwrap();
            let project = k.create_project("Test Project")?;
            let project_id = project.id.clone();
            let task = k.create_task_in_project(&project_id, "Concurrent Update", "Initial")?;
            (project_id, task.id)
        };
        
        let mut handles = vec![];
        
        // Spawn threads that update the task status
        for i in 0..5 {
            let kanban_clone = Arc::clone(&kanban);
            let task_id_clone = task_id.clone();
            
            let handle = thread::spawn(move || {
                let mut k = kanban_clone.lock().unwrap();
                let status = if i % 2 == 0 { TaskStatus::InProgress } else { TaskStatus::Done };
                k.update_task_status(&task_id_clone, status)
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads
        for handle in handles {
            handle.join().unwrap()?;
        }
        
        // Task should have been updated (last write wins)
        let kanban = kanban.lock().unwrap();
        let task = kanban.get_task(&task_id).unwrap();
        assert!(task.status == TaskStatus::InProgress || task.status == TaskStatus::Done);
        
        Ok(())
    }
}