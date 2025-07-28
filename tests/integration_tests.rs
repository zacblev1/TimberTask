/// Integration tests for TimberTask
mod common;

use common::*;
use timber_task::state::{
    timer_state::TimerState,
    kanban_state::{KanbanState, TaskStatus},
    notes_state::NotesState,
};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;
use anyhow::Result;

#[test]
fn test_full_workflow() -> Result<()> {
    let fixture = TestFixture::new();
    
    // Create states
    let mut kanban = fixture.create_kanban_state();
    let mut notes = fixture.create_notes_state();
    let timer = Arc::new(Mutex::new(TimerState::default()));
    
    // 1. Create a project
    let project = kanban.create_project("My Project")?;
    kanban.set_selected_project(&project.id)?;
    
    // 2. Create tasks
    let task1 = kanban.create_task_in_project(&project.id, "Implement feature", "Add new functionality")?;
    let task2 = kanban.create_task_in_project(&project.id, "Write tests", "Ensure quality")?;
    let task3 = kanban.create_task_in_project(&project.id, "Update docs", "Document changes")?;
    
    // 3. Create notes for the project
    let project_notes = notes.create_note("Project Notes", "Notes for My Project", None)?;
    let meeting_notes = notes.create_note("Meeting Notes", "Discussed requirements", Some(&project_notes.id))?;
    let tech_notes = notes.create_note("Technical Decisions", "Architecture choices", Some(&project_notes.id))?;
    
    // 4. Add tags to notes
    let important_tag = notes.create_tag("Important", Some("#FF0000"))?;
    let tech_tag = notes.create_tag("Technical", Some("#0080FF"))?;
    
    notes.add_tag_to_note(&meeting_notes.id, &important_tag.id)?;
    notes.add_tag_to_note(&tech_notes.id, &tech_tag.id)?;
    
    // 5. Start working on a task with timer
    kanban.update_task_status(&task1.id, TaskStatus::InProgress)?;
    {
        let mut t = timer.lock().unwrap();
        t.set_current_task(Some(task1.id.clone()));
        t.start();
    }
    
    // Simulate work
    std::thread::sleep(Duration::from_millis(100));
    
    // 6. Complete the task
    {
        let mut t = timer.lock().unwrap();
        t.pause();
        let elapsed = t.get_remaining_seconds();
        assert!(elapsed < t.work_duration.as_secs());
    }
    
    kanban.update_task_status(&task1.id, TaskStatus::Done)?;
    kanban.add_time_to_task(&task1.id, 100)?; // Add 100 seconds
    
    // 7. Save everything
    kanban.save_to_disk()?;
    notes.save_to_disk()?;
    
    // 8. Load in new instances and verify
    let mut new_kanban = fixture.create_kanban_state();
    let mut new_notes = fixture.create_notes_state();
    
    new_kanban.load_from_disk()?;
    new_notes.load_from_disk()?;
    
    // Verify kanban data
    let loaded_project = &new_kanban.projects[&project.id];
    assert_eq!(loaded_project.name, "My Project");
    assert_eq!(loaded_project.tasks.len(), 3);
    
    let loaded_task = &new_kanban.tasks[&task1.id];
    assert_eq!(loaded_task.status, TaskStatus::Done);
    assert_eq!(loaded_task.time_spent, 100);
    
    // Verify notes data
    let loaded_project_notes = &new_notes.notes[&project_notes.id];
    assert_eq!(loaded_project_notes.children.len(), 2);
    
    let loaded_meeting_notes = &new_notes.notes[&meeting_notes.id];
    assert!(loaded_meeting_notes.tags.contains(&important_tag.id));
    
    Ok(())
}

#[test]
fn test_concurrent_state_updates() -> Result<()> {
    let fixture = TestFixture::new();
    
    // Create shared states
    let kanban = Arc::new(Mutex::new(fixture.create_kanban_state()));
    let notes = Arc::new(Mutex::new(fixture.create_notes_state()));
    let timer = Arc::new(Mutex::new(TimerState::default()));
    
    // Create initial data
    let project_id = {
        let mut k = kanban.lock().unwrap();
        let project = k.create_project("Concurrent Project")?;
        project.id
    };
    
    let note_id = {
        let mut n = notes.lock().unwrap();
        let note = n.create_note("Root Note", "Content", None)?;
        note.id
    };
    
    let mut handles = vec![];
    
    // Thread 1: Create tasks
    let kanban1 = Arc::clone(&kanban);
    let project_id1 = project_id.clone();
    handles.push(thread::spawn(move || {
        let mut k = kanban1.lock().unwrap();
        for i in 0..5 {
            k.create_task_in_project(&project_id1, &format!("Task {}", i), "Description").unwrap();
        }
    }));
    
    // Thread 2: Create notes
    let notes2 = Arc::clone(&notes);
    let note_id2 = note_id.clone();
    handles.push(thread::spawn(move || {
        let mut n = notes2.lock().unwrap();
        for i in 0..5 {
            n.create_note(&format!("Child {}", i), "Content", Some(&note_id2)).unwrap();
        }
    }));
    
    // Thread 3: Toggle timer
    let timer3 = Arc::clone(&timer);
    handles.push(thread::spawn(move || {
        for _ in 0..5 {
            let mut t = timer3.lock().unwrap();
            if t.is_running {
                t.pause();
            } else {
                t.start();
            }
            drop(t);
            thread::sleep(Duration::from_millis(10));
        }
    }));
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify final state
    let k = kanban.lock().unwrap();
    let tasks = k.get_project_tasks(&project_id)?;
    assert_eq!(tasks.len(), 5);
    
    let n = notes.lock().unwrap();
    let parent_note = n.get_note(&note_id).unwrap();
    assert_eq!(parent_note.children.len(), 5);
    
    Ok(())
}

#[test]
fn test_error_recovery() -> Result<()> {
    let fixture = TestFixture::new();
    
    // Test loading from non-existent files
    let mut kanban = fixture.create_kanban_state();
    let mut notes = fixture.create_notes_state();
    
    // Should not error when files don't exist
    kanban.load_from_disk()?;
    notes.load_from_disk()?;
    
    // States should be empty but valid
    assert!(kanban.projects.is_empty());
    assert!(notes.notes.is_empty());
    
    // Test corrupted data recovery
    let corrupt_path = fixture.data_path().join("kanban_data.json");
    std::fs::write(&corrupt_path, "{ invalid json")?;
    
    // Loading should fail gracefully
    assert!(kanban.load_from_disk().is_err());
    
    // State should remain unchanged
    assert!(kanban.projects.is_empty());
    
    Ok(())
}

#[test]
fn test_cross_module_references() -> Result<()> {
    let fixture = TestFixture::new();
    
    let mut kanban = fixture.create_kanban_state();
    let mut notes = fixture.create_notes_state();
    
    // Create a project and tasks
    let project = kanban.create_project("Development Project")?;
    let task = kanban.create_task_in_project(&project.id, "Implement feature", "Description")?;
    
    // Create a note that references the task
    let task_note = notes.create_note(
        &format!("Notes for task: {}", task.title),
        &format!("Task ID: {}\nImplementation details...", task.id),
        None
    )?;
    
    // Create a tag for task-related notes
    let task_tag = notes.create_tag("task-reference", Some("#00FF00"))?;
    notes.add_tag_to_note(&task_note.id, &task_tag.id)?;
    
    // Search for notes related to the task
    let search_results = notes.search_notes(&task.id);
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].id, task_note.id);
    
    // Get all notes with task-reference tag
    let task_notes = notes.get_notes_with_tag(&task_tag.id);
    assert_eq!(task_notes.len(), 1);
    
    Ok(())
}

#[test]
fn test_performance_with_large_dataset() -> Result<()> {
    let fixture = TestFixture::new();
    
    let mut kanban = fixture.create_kanban_state();
    let mut notes = fixture.create_notes_state();
    
    // Create many projects and tasks
    let start = std::time::Instant::now();
    
    for i in 0..10 {
        let project = kanban.create_project(&format!("Project {}", i))?;
        
        // Create 100 tasks per project
        for j in 0..100 {
            kanban.create_task_in_project(
                &project.id,
                &format!("Task {}-{}", i, j),
                "Description"
            )?;
        }
    }
    
    let kanban_creation_time = start.elapsed();
    println!("Created 1000 tasks in {:?}", kanban_creation_time);
    
    // Create many notes
    let start = std::time::Instant::now();
    
    let root = notes.create_note("Root", "Content", None)?;
    for i in 0..100 {
        let parent = notes.create_note(&format!("Section {}", i), "Content", Some(&root.id))?;
        
        for j in 0..10 {
            notes.create_note(
                &format!("Note {}-{}", i, j),
                "Content",
                Some(&parent.id)
            )?;
        }
    }
    
    let notes_creation_time = start.elapsed();
    println!("Created 1001 notes in {:?}", notes_creation_time);
    
    // Test save performance
    let start = std::time::Instant::now();
    kanban.save_to_disk()?;
    notes.save_to_disk()?;
    let save_time = start.elapsed();
    println!("Saved all data in {:?}", save_time);
    
    // Test load performance
    let mut new_kanban = fixture.create_kanban_state();
    let mut new_notes = fixture.create_notes_state();
    
    let start = std::time::Instant::now();
    new_kanban.load_from_disk()?;
    new_notes.load_from_disk()?;
    let load_time = start.elapsed();
    println!("Loaded all data in {:?}", load_time);
    
    // Verify data integrity
    assert_eq!(new_kanban.projects.len(), 10);
    assert_eq!(new_kanban.tasks.len(), 1000);
    assert_eq!(new_notes.notes.len(), 1001);
    
    // Performance assertions (these are generous to account for CI/different hardware)
    assert!(kanban_creation_time < Duration::from_secs(2));
    assert!(notes_creation_time < Duration::from_secs(2));
    assert!(save_time < Duration::from_secs(1));
    assert!(load_time < Duration::from_secs(1));
    
    Ok(())
}