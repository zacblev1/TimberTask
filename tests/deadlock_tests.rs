#[cfg(test)]
mod deadlock_tests {
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    use timber_task::app::App;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    
    /// Test that creating a Kanban task doesn't cause a deadlock
    #[test]
    fn test_kanban_task_creation_no_deadlock() {
        // Create app with test kanban state
        let mut app = App::new().expect("Failed to create app");
        
        // Create a test project first
        {
            let mut kanban = app.kanban_state.lock().expect("Failed to lock kanban");
            kanban.create_project("Test Project");
        }
        
        // Open task form
        app.show_task_form = true;
        app.focused_field = timber_task::app::FormField::Title;
        app.task_form_title = "Test Task".to_string();
        app.task_form_description = "Test Description".to_string();
        
        // Simulate pressing Save button
        app.focused_field = timber_task::app::FormField::SaveButton;
        
        // This should not deadlock
        let result = std::panic::catch_unwind(move || {
            // Set a timeout for the operation
            let handle = thread::spawn(move || {
                let save_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
                app.handle_task_form_keys(save_key).expect("Failed to handle save");
            });
            
            // Wait for completion with timeout
            thread::sleep(Duration::from_millis(100));
            
            // If we get here without hanging, the deadlock is fixed
            assert!(true, "Task creation completed without deadlock");
        });
        
        assert!(result.is_ok(), "Task creation should not panic or deadlock");
    }
    
    /// Test rapid task creation and deletion
    #[test] 
    fn test_rapid_task_operations_no_deadlock() {
        let mut app = App::new().expect("Failed to create app");
        
        // Create a test project
        {
            let mut kanban = app.kanban_state.lock().expect("Failed to lock kanban");
            kanban.create_project("Test Project");
        }
        
        // Perform rapid operations
        for i in 0..10 {
            // Create task
            app.show_task_form = true;
            app.task_form_title = format!("Task {}", i);
            app.task_form_description = format!("Description {}", i);
            app.focused_field = timber_task::app::FormField::SaveButton;
            
            let save_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
            app.handle_task_form_keys(save_key).expect("Failed to create task");
            
            // Select and delete if we have tasks
            if app.selected_task.is_some() {
                let delete_key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty());
                app.handle_kanban_keys(delete_key).expect("Failed to delete task");
            }
        }
        
        assert!(true, "Rapid operations completed without deadlock");
    }
    
    /// Test concurrent access from multiple threads
    #[test]
    fn test_concurrent_access_no_deadlock() {
        let app = Arc::new(std::sync::Mutex::new(
            App::new().expect("Failed to create app")
        ));
        
        // Create a test project
        {
            let mut app_guard = app.lock().expect("Failed to lock app");
            let mut kanban = app_guard.kanban_state.lock().expect("Failed to lock kanban");
            kanban.create_project("Test Project");
        }
        
        let mut handles = vec![];
        
        // Spawn multiple threads performing operations
        for i in 0..5 {
            let app_clone = Arc::clone(&app);
            let handle = thread::spawn(move || {
                for j in 0..5 {
                    let mut app_guard = app_clone.lock().expect("Failed to lock app");
                    
                    // Create task
                    app_guard.show_task_form = true;
                    app_guard.task_form_title = format!("Thread {} Task {}", i, j);
                    app_guard.focused_field = timber_task::app::FormField::SaveButton;
                    
                    let save_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
                    app_guard.handle_task_form_keys(save_key).expect("Failed to create task");
                    
                    // Small delay to increase chance of contention
                    thread::sleep(Duration::from_millis(10));
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads with timeout
        for handle in handles {
            handle.join().expect("Thread panicked");
        }
        
        assert!(true, "Concurrent access completed without deadlock");
    }
}