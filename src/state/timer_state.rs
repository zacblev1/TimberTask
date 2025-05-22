use std::time::{Duration, Instant};

/// Timer state for the Pomodoro timer
pub struct TimerState {
    /// Start time of the current timer (None if not running)
    pub start_time: Option<Instant>,
    /// Duration of the current timer period
    pub duration: Duration,
    /// Whether the timer is currently running
    pub is_running: bool,
    /// Whether the current period is a work period (true) or break period (false)
    pub is_work_period: bool,
    /// Duration of work periods
    pub work_duration: Duration,
    /// Duration of break periods
    pub break_duration: Duration,
    /// Number of completed pomodoros
    pub completed_pomodoros: usize,
    /// ID of the task currently being worked on (if any)
    pub current_task_id: Option<String>,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            start_time: None,
            duration: Duration::from_secs(25 * 60), // Default 25 minutes
            is_running: false,
            is_work_period: true,
            work_duration: Duration::from_secs(25 * 60), // 25 minutes
            break_duration: Duration::from_secs(5 * 60),  // 5 minutes
            completed_pomodoros: 0,
            current_task_id: None,
        }
    }
}

impl TimerState {
    /// Start the timer
    pub fn start(&mut self) {
        if !self.is_running {
            self.start_time = Some(Instant::now());
            self.is_running = true;
        }
    }
    
    /// Pause the timer
    pub fn pause(&mut self) {
        if self.is_running {
            if let Some(start_time) = self.start_time {
                let elapsed = start_time.elapsed();
                
                // Adjust the duration to the remaining time
                if elapsed < self.duration {
                    self.duration -= elapsed;
                } else {
                    self.duration = Duration::from_secs(0);
                }
                
                self.start_time = None;
                self.is_running = false;
            }
        }
    }
    
    /// Reset the timer
    pub fn reset(&mut self) {
        self.start_time = None;
        self.is_running = false;
        self.duration = if self.is_work_period {
            self.work_duration
        } else {
            self.break_duration
        };
    }
    
    /// Switch between work and break periods
    pub fn switch_period(&mut self) {
        // Toggle between work and break
        self.is_work_period = !self.is_work_period;
        
        // If just completed a work period, increment the counter
        if !self.is_work_period {
            self.completed_pomodoros += 1;
        }
        
        // Set the appropriate duration
        self.duration = if self.is_work_period {
            self.work_duration
        } else {
            self.break_duration
        };
        
        // Reset the timer
        self.start_time = None;
        self.is_running = false;
    }
    
    /// Set work/break period
    pub fn set_work_period(&mut self, is_work: bool) {
        if self.is_work_period != is_work {
            self.is_work_period = is_work;
            self.duration = if is_work {
                self.work_duration
            } else {
                self.break_duration
            };
            self.start_time = None;
            self.is_running = false;
        }
    }
    
    /// Update timer settings
    #[allow(dead_code)]
    pub fn update_settings(&mut self, work_minutes: u64, break_minutes: u64) {
        self.work_duration = Duration::from_secs(work_minutes * 60);
        self.break_duration = Duration::from_secs(break_minutes * 60);
        
        // Update current duration if not running
        if !self.is_running {
            self.duration = if self.is_work_period {
                self.work_duration
            } else {
                self.break_duration
            };
        }
    }
    
    /// Get the remaining seconds on the timer
    pub fn get_remaining_seconds(&self) -> u64 {
        if let Some(start_time) = self.start_time {
            let elapsed = start_time.elapsed();
            if elapsed < self.duration {
                (self.duration - elapsed).as_secs()
            } else {
                0
            }
        } else {
            self.duration.as_secs()
        }
    }
    
    /// Check if the timer is complete
    pub fn is_complete(&self) -> bool {
        if self.is_running {
            if let Some(start_time) = self.start_time {
                start_time.elapsed() >= self.duration
            } else {
                false
            }
        } else {
            false
        }
    }
    
    /// Complete the current period
    pub fn complete_period(&mut self) {
        self.switch_period();
        
        // Could also trigger a notification here
        // notify_rust::Notification::new()
        //     .summary("Pomodoro Timer")
        //     .body(&format!("{} period complete!", if self.is_work_period { "Break" } else { "Work" }))
        //     .show()
        //     .unwrap();
    }
    
    /// Set the current task being worked on
    pub fn set_current_task(&mut self, task_id: Option<String>) {
        self.current_task_id = task_id;
    }
    
    /// Update timer on tick
    pub fn tick(&mut self) {
        // Only used for periodic updates in the app tick loop
        // No implementation needed here as we calculate remaining time on demand
    }
}