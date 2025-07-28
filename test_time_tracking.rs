use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use timber_task::state::kanban_state::KanbanState;
use timber_task::state::timer_state::TimerState;
use timber_task::state::save_request::SaveRequest;

fn main() {
    println!("Testing time tracking functionality...\n");
    
    // Create states
    let kanban_state = Arc::new(Mutex::new(KanbanState::default()));
    let timer_state = Arc::new(Mutex::new(TimerState::default()));
    
    // Create a project and task
    let (project_id, task_id) = {
        let mut kanban = kanban_state.lock().unwrap();
        let (project, _) = kanban.create_project("Test Project").unwrap();
        let project_id = project.id.clone();
        let (task, _) = kanban.create_task_in_project(&project_id, "Test Task", "Testing time tracking").unwrap();
        println!("Created task '{}' with ID: {}", task.title, task.id);
        println!("Initial time_spent: {} seconds", task.time_spent);
        (project_id, task.id.clone())
    };
    
    // Set task for timer
    {
        let mut timer = timer_state.lock().unwrap();
        timer.set_current_task(Some(task_id.clone()));
        println!("\nAssigned task {} to timer", task_id);
    }
    
    // Start timer
    {
        let mut timer = timer_state.lock().unwrap();
        timer.start();
        println!("Started timer");
    }
    
    // Wait for timer to complete (using debug timer - 10 seconds)
    println!("Waiting 10 seconds for timer to complete...");
    thread::sleep(Duration::from_secs(11));
    
    // Check if timer is complete
    {
        let mut timer = timer_state.lock().unwrap();
        let remaining = timer.get_remaining_seconds();
        println!("\nTimer remaining seconds: {}", remaining);
        
        if remaining == 0 {
            println!("Timer completed!");
            
            // Record the time
            let was_work_period = timer.is_work_period;
            let current_task_id = timer.current_task_id.clone();
            let elapsed = timer.work_duration.as_secs();
            
            println!("Was work period: {}", was_work_period);
            println!("Current task ID: {:?}", current_task_id);
            println!("Elapsed seconds: {}", elapsed);
            
            if was_work_period && current_task_id.is_some() {
                // Complete the period
                timer.complete_period();
                
                // Add time to task
                let task_id = current_task_id.unwrap();
                let mut kanban = kanban_state.lock().unwrap();
                match kanban.add_time_to_task(&task_id, elapsed) {
                    Ok((task, _)) => {
                        println!("\nSuccessfully added {} seconds to task '{}'", elapsed, task.title);
                        println!("New time_spent: {} seconds", task.time_spent);
                    }
                    Err(e) => {
                        println!("\nERROR adding time to task: {}", e);
                    }
                }
            }
        }
    }
    
    // Final check
    {
        let kanban = kanban_state.lock().unwrap();
        if let Some(task) = kanban.get_task(&task_id) {
            println!("\n=== FINAL RESULT ===");
            println!("Task: {}", task.title);
            println!("Time spent: {} seconds", task.time_spent);
            if task.time_spent > 0 {
                println!("✅ TIME TRACKING WORKS!");
            } else {
                println!("❌ TIME NOT RECORDED");
            }
        }
    }
}