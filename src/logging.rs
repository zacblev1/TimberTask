use anyhow::Result;
use std::path::PathBuf;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Initialize the tracing subscriber for logging
pub fn init_logging() -> Result<()> {
    // Get the log directory
    let log_dir = home::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?
        .join(".timber-task")
        .join("logs");
    
    // Create log directory if it doesn't exist
    std::fs::create_dir_all(&log_dir)?;
    
    // Create a rolling file appender
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,  // Create a new log file daily
        log_dir,
        "timber-task.log",
    );
    
    // Create the file layer
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)  // No ANSI colors in file output
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true);
    
    // For TUI applications, we should never log to console as it interferes with the UI
    // All logs go to file only
    
    // Set up the subscriber with environment filter
    // This allows users to control log levels via RUST_LOG env var
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            // Default log levels
            #[cfg(debug_assertions)]
            return EnvFilter::new("timber_task=debug,info");
            
            #[cfg(not(debug_assertions))]
            return EnvFilter::new("timber_task=info,warn");
        });
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .init();
    
    tracing::info!("TimberTask logging initialized");
    
    Ok(())
}

/// Get the path to the current log file
pub fn get_log_file_path() -> Result<PathBuf> {
    let log_dir = home::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?
        .join(".timber-task")
        .join("logs");
    
    // Get today's date for the log file name
    let today = chrono::Local::now().format("%Y-%m-%d");
    let log_file = log_dir.join(format!("timber-task.{}.log", today));
    
    Ok(log_file)
}