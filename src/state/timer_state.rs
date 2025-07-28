use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Result};
use tracing::info;
use crate::utils::atomic_save::{atomic_write, atomic_read};
use crate::state::save_request::SaveRequest;

/// Serializable timer state data
#[derive(Serialize, Deserialize, Clone)]
struct TimerStateData {
    /// When the timer was started (as unix timestamp)
    start_timestamp: Option<u64>,
    /// Remaining duration in seconds
    remaining_seconds: u64,
    /// Whether the timer is currently running
    is_running: bool,
    /// Whether the current period is a work period
    is_work_period: bool,
    /// Work duration in seconds
    work_seconds: u64,
    /// Break duration in seconds  
    break_seconds: u64,
    /// Number of completed pomodoros
    completed_pomodoros: usize,
    /// ID of the task currently being worked on
    current_task_id: Option<String>,
}

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
    /// Path to the data file
    data_file_path: PathBuf,
}

impl Default for TimerState {
    fn default() -> Self {
        let data_file_path = home::home_dir()
            .map(|home| home.join(".timber-task").join("timer_state.json"))
            .unwrap_or_else(|| {
                std::env::temp_dir().join("timber-task").join("timer_state.json")
            });
            
        // Use shorter durations for testing if DEBUG_TIMER env var is set
        let (work_duration, break_duration) = if std::env::var("DEBUG_TIMER").is_ok() {
            (Duration::from_secs(10), Duration::from_secs(5)) // 10 seconds work, 5 seconds break for testing
        } else {
            (Duration::from_secs(25 * 60), Duration::from_secs(5 * 60)) // Normal: 25 min work, 5 min break
        };
            
        Self {
            start_time: None,
            duration: work_duration,
            is_running: false,
            is_work_period: true,
            work_duration,
            break_duration,
            completed_pomodoros: 0,
            current_task_id: None,
            data_file_path,
        }
    }
}

impl TimerState {
    /// Start the timer
    pub fn start(&mut self) -> SaveRequest {
        if !self.is_running {
            self.start_time = Some(Instant::now());
            self.is_running = true;
            info!("Timer started: {} period for {} seconds, tracking task: {:?}", 
                if self.is_work_period { "work" } else { "break" },
                self.duration.as_secs(),
                self.current_task_id);
        }
        SaveRequest::Full
    }
    
    /// Pause the timer
    pub fn pause(&mut self) -> SaveRequest {
        if self.is_running {
            info!("Timer paused");
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
        SaveRequest::Full
    }
    
    /// Reset the timer
    pub fn reset(&mut self) -> SaveRequest {
        self.start_time = None;
        self.is_running = false;
        self.duration = if self.is_work_period {
            self.work_duration
        } else {
            self.break_duration
        };
        SaveRequest::Full
    }
    
    /// Switch between work and break periods
    pub fn switch_period(&mut self) -> SaveRequest {
        let was_work_period = self.is_work_period;
        
        // Toggle between work and break
        self.is_work_period = !self.is_work_period;
        
        // If just completed a work period, increment the counter
        if was_work_period && !self.is_work_period {
            self.completed_pomodoros += 1;
            info!("Completed pomodoro #{}", self.completed_pomodoros);
        } else {
            info!("Break period completed, starting work period");
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
        
        // Don't clear task ID here - let the app handle it after recording time
        // The app's tick() method needs the task ID to record time spent
        if was_work_period && !self.is_work_period && self.current_task_id.is_some() {
            info!("Keeping task ID for time recording: {:?}", self.current_task_id);
            // Note: The app will clear this after recording the time
        }
        
        info!("Switched to {} period, duration: {} seconds", 
             if self.is_work_period { "work" } else { "break" },
             self.duration.as_secs());
        SaveRequest::Full
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
    pub fn complete_period(&mut self) -> SaveRequest {
        self.switch_period()
    }
    
    /// Set the current task being worked on
    pub fn set_current_task(&mut self, task_id: Option<String>) -> SaveRequest {
        info!("Setting current task for timer: {:?}", task_id);
        self.current_task_id = task_id;
        SaveRequest::Full
    }
    
    /// Update timer on tick
    pub fn tick(&mut self) -> SaveRequest {
        // Increment tick counter
        static mut TICK_COUNT: u64 = 0;
        unsafe {
            TICK_COUNT += 1;
            
            // Save every 10 ticks (approx 2.5 seconds) when running
            if self.is_running && TICK_COUNT % 10 == 0 {
                return SaveRequest::Full;
            }
        }
        
        SaveRequest::None
    }
    
    /// Save the timer state to disk
    pub fn save_to_disk(&self) -> Result<()> {
        let data = TimerStateData {
            start_timestamp: self.start_time.map(|start| {
                // Calculate when the timer was started based on current time and elapsed
                let elapsed = start.elapsed();
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                now.saturating_sub(elapsed.as_secs())
            }),
            remaining_seconds: self.get_remaining_seconds(),
            is_running: self.is_running,
            is_work_period: self.is_work_period,
            work_seconds: self.work_duration.as_secs(),
            break_seconds: self.break_duration.as_secs(),
            completed_pomodoros: self.completed_pomodoros,
            current_task_id: self.current_task_id.clone(),
        };
        
        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| anyhow!("Failed to serialize timer state: {}", e))?;
        
        atomic_write(&self.data_file_path, &json)
            .map_err(|e| anyhow!("Failed to write timer state to disk: {}", e))?;
            
        Ok(())
    }
    
    /// Process a save request outside of mutex locks
    pub fn process_save_request(&self, request: &SaveRequest) -> Result<()> {
        match request {
            SaveRequest::Full => self.save_to_disk(),
            SaveRequest::None => Ok(()),
        }
    }
    
    /// Load the timer state from disk
    pub fn load_from_disk(&mut self) -> Result<()> {
        if !self.data_file_path.exists() {
            info!("No timer state file found, using defaults");
            return Ok(());
        }
        
        let json = atomic_read(&self.data_file_path)
            .map_err(|e| anyhow!("Failed to read timer state from disk: {}", e))?;
            
        let data: TimerStateData = serde_json::from_str(&json)
            .map_err(|e| anyhow!("Failed to deserialize timer state: {}", e))?;
            
        // Restore the state
        self.is_running = data.is_running;
        self.is_work_period = data.is_work_period;
        
        // Override durations if DEBUG_TIMER is set
        if std::env::var("DEBUG_TIMER").is_ok() {
            self.work_duration = Duration::from_secs(10);
            self.break_duration = Duration::from_secs(5);
        } else {
            self.work_duration = Duration::from_secs(data.work_seconds);
            self.break_duration = Duration::from_secs(data.break_seconds);
        }
        
        self.completed_pomodoros = data.completed_pomodoros;
        self.current_task_id = data.current_task_id;
        
        // Restore the timer if it was running
        if data.is_running {
            if let Some(start_ts) = data.start_timestamp {
                // Calculate how much time has passed since the timer was started
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                    
                let elapsed_seconds = now.saturating_sub(start_ts);
                
                // Determine the original duration
                let original_duration = if data.is_work_period {
                    data.work_seconds
                } else {
                    data.break_seconds
                };
                
                if elapsed_seconds < original_duration {
                    // Timer should still be running
                    self.duration = Duration::from_secs(original_duration - elapsed_seconds);
                    self.start_time = Some(Instant::now() - Duration::from_secs(elapsed_seconds));
                    self.is_running = true;
                } else {
                    // Timer has expired while we were away
                    self.duration = Duration::from_secs(0);
                    self.is_running = false;
                    self.start_time = None;
                }
            }
        } else {
            // Timer was not running, just restore the remaining duration
            self.duration = Duration::from_secs(data.remaining_seconds);
            self.start_time = None;
        }
        
        info!("Timer state loaded from disk");
        Ok(())
    }
}