use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use tracing::{info, error};

use super::App;
use crate::utils::mutex::lock_mutex;

impl App {
    /// Handle timer tab key inputs
    pub fn handle_timer_keys(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('s') | KeyCode::Char('S') => {
                let save_request = {
                    let mut timer = lock_mutex(&self.timer_state)?;
                    timer.start()
                };
                // Process save request outside of mutex lock
                if save_request.is_needed() {
                    let timer = lock_mutex(&self.timer_state)?;
                    timer.process_save_request(&save_request)?;
                }
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                let save_request = {
                    let mut timer = lock_mutex(&self.timer_state)?;
                    timer.pause()
                };
                // Process save request outside of mutex lock
                if save_request.is_needed() {
                    let timer = lock_mutex(&self.timer_state)?;
                    timer.process_save_request(&save_request)?;
                }
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                let save_request = {
                    let mut timer = lock_mutex(&self.timer_state)?;
                    timer.reset()
                };
                // Process save request outside of mutex lock
                if save_request.is_needed() {
                    let timer = lock_mutex(&self.timer_state)?;
                    timer.process_save_request(&save_request)?;
                }
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                let save_request = {
                    let mut timer = lock_mutex(&self.timer_state)?;
                    timer.is_work_period = !timer.is_work_period;
                    timer.reset()
                };
                // Process save request outside of mutex lock
                if save_request.is_needed() {
                    let timer = lock_mutex(&self.timer_state)?;
                    timer.process_save_request(&save_request)?;
                }
            }
            KeyCode::Char(' ') => {
                // Space toggles start/pause
                let save_request = {
                    let mut timer = lock_mutex(&self.timer_state)?;
                    if timer.is_running {
                        timer.pause()
                    } else {
                        timer.start()
                    }
                };
                // Process save request outside of mutex lock
                if save_request.is_needed() {
                    let timer = lock_mutex(&self.timer_state)?;
                    timer.process_save_request(&save_request)?;
                }
            }
            KeyCode::Char('k') => {
                // Skip current period
                let (timer_save_request, was_work_period, task_id, elapsed_seconds) = {
                    let mut timer = lock_mutex(&self.timer_state)?;
                    let was_work = timer.is_work_period;
                    let task_id = timer.current_task_id.clone();
                    
                    // Calculate actual elapsed time
                    let elapsed = if timer.is_running && timer.start_time.is_some() {
                        let start_time = timer.start_time.unwrap();
                        start_time.elapsed().as_secs()
                    } else {
                        // If timer wasn't running, use the full duration minus remaining
                        let full_duration = if was_work {
                            timer.work_duration.as_secs()
                        } else {
                            timer.break_duration.as_secs()
                        };
                        full_duration - timer.get_remaining_seconds()
                    };
                    
                    let save_req = timer.complete_period();
                    (save_req, was_work, task_id, elapsed)
                };
                
                // Process timer save request outside of mutex lock
                if timer_save_request.is_needed() {
                    let timer = lock_mutex(&self.timer_state)?;
                    timer.process_save_request(&timer_save_request)?;
                }
                
                // If a task was being tracked and it was a work period, log time
                if let Some(task_id) = task_id {
                    if was_work_period && elapsed_seconds > 0 {
                        info!("Skip key pressed: Adding {} seconds to task {}", elapsed_seconds, task_id);
                        let save_request = {
                            let mut kanban = lock_mutex(&self.kanban_state)?;
                            match kanban.add_time_to_task(&task_id, elapsed_seconds) {
                                Ok((task, save_req)) => {
                                    info!("Successfully added {} seconds to task '{}'. Total time: {} seconds", 
                                         elapsed_seconds, task.title, task.time_spent);
                                    Some(save_req)
                                },
                                Err(e) => {
                                    error!("Failed to add time to task: {}", e);
                                    None
                                },
                            }
                        }; // Lock is dropped here
                        
                        // Process save request outside of mutex lock
                        if let Some(save_req) = save_request {
                            let kanban = lock_mutex(&self.kanban_state)?;
                            let _ = kanban.process_save_request(&save_req);
                        }
                    }
                }
            }
            _ => {}
        }
        
        Ok(false)
    }
}