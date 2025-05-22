use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Ensure a directory exists, creating it if needed
#[allow(dead_code)]
pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)
            .map_err(|e| anyhow!("Failed to create directory {}: {}", path.display(), e))?;
    }
    Ok(())
}

/// Get the application data directory
#[allow(dead_code)]
pub fn get_data_dir() -> Result<PathBuf> {
    let home_dir = home::home_dir()
        .ok_or_else(|| anyhow!("Failed to get home directory"))?;
    let data_dir = home_dir.join(".timber-task");
    ensure_dir_exists(&data_dir)?;
    Ok(data_dir)
}

/// Get a path to a file in the application data directory
#[allow(dead_code)]
pub fn get_data_file(file_name: &str) -> Result<PathBuf> {
    let data_dir = get_data_dir()?;
    Ok(data_dir.join(file_name))
}