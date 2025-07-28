use anyhow::Result;
use std::thread;
use std::time::Duration;
use tracing::{warn, error};

/// Retry configuration for operations that may fail transiently
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Factor to multiply delay by after each attempt
    pub backoff_factor: f32,
    /// Maximum delay between retries
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            backoff_factor: 2.0,
            max_delay: Duration::from_secs(5),
        }
    }
}

/// Retry an operation with exponential backoff
pub fn retry_with_backoff<T, E, F>(
    mut operation: F,
    config: RetryConfig,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Display,
{
    let mut delay = config.initial_delay;
    
    for attempt in 1..=config.max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if attempt < config.max_attempts => {
                warn!(
                    "Operation failed (attempt {}/{}): {}. Retrying in {:?}",
                    attempt, config.max_attempts, e, delay
                );
                thread::sleep(delay);
                
                // Calculate next delay with backoff
                delay = Duration::from_secs_f32(
                    (delay.as_secs_f32() * config.backoff_factor)
                        .min(config.max_delay.as_secs_f32())
                );
            }
            Err(e) => {
                error!(
                    "Operation failed after {} attempts: {}",
                    config.max_attempts, e
                );
                return Err(e);
            }
        }
    }
    
    unreachable!()
}

/// Execute an operation with a fallback value on failure
pub fn with_fallback<T, E, F, G>(
    operation: F,
    fallback: G,
) -> T
where
    F: FnOnce() -> Result<T, E>,
    G: FnOnce(E) -> T,
    E: std::fmt::Display,
{
    match operation() {
        Ok(value) => value,
        Err(e) => {
            warn!("Operation failed, using fallback: {}", e);
            fallback(e)
        }
    }
}

/// Load data from disk with automatic fallback to default
pub fn load_or_default<T, P, F, D>(
    path: P,
    loader: F,
    default: D,
) -> T
where
    P: AsRef<std::path::Path>,
    F: FnOnce(&std::path::Path) -> Result<T>,
    D: FnOnce() -> T,
{
    let path = path.as_ref();
    
    match loader(path) {
        Ok(data) => {
            tracing::info!("Successfully loaded data from {:?}", path);
            data
        }
        Err(e) => {
            warn!("Failed to load from {:?}, using default: {}", path, e);
            default()
        }
    }
}

/// Safe partial save - attempts to save multiple components and reports all failures
pub struct PartialSaveResult {
    pub successes: Vec<String>,
    pub failures: Vec<(String, String)>,
}

impl PartialSaveResult {
    pub fn all_succeeded(&self) -> bool {
        self.failures.is_empty()
    }
    
    pub fn log_summary(&self) {
        if self.all_succeeded() {
            tracing::info!("All components saved successfully: {:?}", self.successes);
        } else {
            error!(
                "Partial save failure. Succeeded: {:?}, Failed: {:?}",
                self.successes, self.failures
            );
        }
    }
}

/// Execute multiple save operations, collecting all results
pub fn partial_save<I, F>(operations: I) -> PartialSaveResult
where
    I: IntoIterator<Item = (String, F)>,
    F: FnOnce() -> Result<()>,
{
    let mut result = PartialSaveResult {
        successes: Vec::new(),
        failures: Vec::new(),
    };
    
    for (name, operation) in operations {
        match operation() {
            Ok(_) => result.successes.push(name),
            Err(e) => result.failures.push((name, e.to_string())),
        }
    }
    
    result.log_summary();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    
    #[test]
    fn test_retry_succeeds_on_second_attempt() {
        let mut attempt = 0;
        let result = retry_with_backoff(
            || {
                attempt += 1;
                if attempt < 2 {
                    Err(anyhow!("Temporary failure"))
                } else {
                    Ok(42)
                }
            },
            RetryConfig {
                max_attempts: 3,
                initial_delay: Duration::from_millis(10),
                ..Default::default()
            },
        );
        
        assert_eq!(result.unwrap(), 42);
    }
    
    #[test]
    fn test_with_fallback_uses_fallback_on_error() {
        let result = with_fallback(
            || Err::<i32, _>(anyhow!("Operation failed")),
            |_| 100,
        );
        
        assert_eq!(result, 100);
    }
    
    #[test]
    fn test_partial_save_reports_mixed_results() {
        let operations: Vec<(String, Box<dyn FnOnce() -> Result<()>>)> = vec![
            ("component1".to_string(), Box::new(|| Ok(()))),
            ("component2".to_string(), Box::new(|| Err(anyhow!("Save failed")))),
            ("component3".to_string(), Box::new(|| Ok(()))),
        ];
        
        let result = partial_save(operations);
        
        assert_eq!(result.successes.len(), 2);
        assert_eq!(result.failures.len(), 1);
        assert!(!result.all_succeeded());
    }
}