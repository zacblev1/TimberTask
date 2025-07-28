#[cfg(test)]
mod tests {
    use crate::app::{App, FormField, NoteField};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use crate::state::kanban_state::TaskStatus;
    use crate::utils::mutex::lock_mutex;
    use anyhow::Result;

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
            assert_eq!(in_progress_tasks[0].title, "Movable Task", "Task should be in In Progress");
        }
        
        // Move task to Done
        app.handle_key(create_key_event(KeyCode::Char('D'))).unwrap();
        
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
            assert_eq!(done_tasks[0].title, "Movable Task", "Task should be in Done");
        }
    }

    #[test]
    fn test_space_key_connects_task_to_timer() -> Result<()> {
        let mut app = App::new()?;
        
        // Create a test project and task
        let task_id = {
            let mut kanban = lock_mutex(&app.kanban_state)?;
            
            // Create and select a project
            let (project, _) = kanban.create_project("Test Project")?;
            kanban.selected_project_id = Some(project.id.clone());
            
            // Create a task
            let (task, _) = kanban.create_task_in_project(&project.id, "Test Task", "Test Description")?;
            let task_id = task.id.clone();
            
            // Move task to InProgress
            let (_task, _) = kanban.update_task_status(&task_id, TaskStatus::InProgress)?;
            
            task_id
        };
        
        // Select the task in the kanban board (column 1 = InProgress, row 0 = first task)
        app.selected_task = Some((1, 0));
        
        // Verify initial state
        {
            let timer = lock_mutex(&app.timer_state)?;
            assert!(timer.current_task_id.is_none(), "Timer should not have a task initially");
        }
        
        // Simulate pressing Space key to toggle task tracking
        let key_event = create_key_event(KeyCode::Char(' '));
        app.handle_kanban_keys(key_event)?;
        
        // Verify timer state after toggle
        {
            let timer = lock_mutex(&app.timer_state)?;
            assert_eq!(timer.current_task_id, Some(task_id.clone()), "Timer should now track the selected task");
            assert!(timer.is_running, "Timer should be running after selecting a task");
        }
        
        // Toggle again to stop tracking
        let key_event = create_key_event(KeyCode::Char(' '));
        app.handle_kanban_keys(key_event)?;
        
        // Verify timer state after second toggle
        {
            let timer = lock_mutex(&app.timer_state)?;
            assert!(timer.current_task_id.is_none(), "Timer should stop tracking the task");
        }
        
        Ok(())
    }

    #[test]
    fn test_get_selected_task_id() -> Result<()> {
        let mut app = App::new()?;
        
        // Create a test project and task
        let task_id = {
            let mut kanban = lock_mutex(&app.kanban_state)?;
            
            // Create and select a project
            let (project, _) = kanban.create_project("Test Project")?;
            kanban.selected_project_id = Some(project.id.clone());
            
            // Create a task in InProgress
            let (task, _) = kanban.create_task_in_project(&project.id, "Test Task", "Test Description")?;
            let task_id = task.id.clone();
            let (_task, _) = kanban.update_task_status(&task_id, TaskStatus::InProgress)?;
            
            task_id
        };
        
        // Initially no task is selected
        assert!(app.get_selected_task_id().is_none());
        
        // Select the task
        app.selected_task = Some((1, 0));  // Column 1 = InProgress, row 0
        
        // Verify we get the correct task ID
        let selected_id = app.get_selected_task_id();
        assert_eq!(selected_id, Some(task_id));
        
        Ok(())
    }

    #[test]
    fn test_space_key_only_works_in_progress_column() -> Result<()> {
        let mut app = App::new()?;
        
        // Create tasks in different columns
        {
            let mut kanban = lock_mutex(&app.kanban_state)?;
            
            let (project, _) = kanban.create_project("Test Project")?;
            kanban.selected_project_id = Some(project.id.clone());
            
            // Create a todo task
            kanban.create_task_in_project(&project.id, "Todo Task", "Test")?;
            
            // Create an in-progress task
            let (task, _) = kanban.create_task_in_project(&project.id, "InProgress Task", "Test")?;
            kanban.update_task_status(&task.id, TaskStatus::InProgress)?;
        }
        
        // Try to track a task in Todo column (should not work)
        app.selected_task = Some((0, 0));  // Column 0 = Todo
        let key_event = create_key_event(KeyCode::Char(' '));
        app.handle_kanban_keys(key_event)?;
        
        {
            let timer = lock_mutex(&app.timer_state)?;
            assert!(timer.current_task_id.is_none(), "Should not track tasks from Todo column");
        }
        
        // Select task in InProgress column (should work)
        app.selected_task = Some((1, 0));  // Column 1 = InProgress
        let key_event = create_key_event(KeyCode::Char(' '));
        app.handle_kanban_keys(key_event)?;
        
        {
            let timer = lock_mutex(&app.timer_state)?;
            assert!(timer.current_task_id.is_some(), "Should track tasks from InProgress column");
        }
        
        Ok(())
    }
}