use chrono::{Local, TimeZone};

/// Format seconds as MM:SS
pub fn format_time(seconds: u64) -> String {
    let minutes = seconds / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}", minutes, secs)
}

/// Format time spent in a human-readable format
#[allow(dead_code)]
pub fn format_time_spent(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Format a Unix timestamp as a human-readable date/time
pub fn format_timestamp(timestamp: u64) -> String {
    let dt = Local.timestamp_opt(timestamp as i64, 0).unwrap();
    dt.format("%Y-%m-%d %H:%M").to_string()
}