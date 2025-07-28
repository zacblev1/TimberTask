use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;
use anyhow::{Context, Result};
use uuid::Uuid;

/// Atomically write data to a file by writing to a temporary file first,
/// then renaming it to the target path. This ensures that the file is either
/// fully written or not written at all, preventing corruption.
pub fn atomic_write<P: AsRef<Path>>(path: P, data: &str) -> Result<()> {
    let path = path.as_ref();
    
    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory for {:?}", path))?;
    }
    
    // Generate a unique temporary file name in the same directory
    let temp_path = generate_temp_path(path)?;
    
    // Write to the temporary file
    let result = write_temp_file(&temp_path, data);
    
    // If writing failed, clean up the temp file
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
        return result;
    }
    
    // Atomically rename the temp file to the target path
    atomic_rename(&temp_path, path)?;
    
    Ok(())
}

/// Generate a temporary file path in the same directory as the target
fn generate_temp_path(target: &Path) -> Result<PathBuf> {
    let parent = target.parent()
        .ok_or_else(|| anyhow::anyhow!("Target path has no parent directory"))?;
    
    let file_name = target.file_name()
        .ok_or_else(|| anyhow::anyhow!("Target path has no file name"))?;
    
    let temp_name = format!(".{}.{}.tmp", file_name.to_string_lossy(), Uuid::new_v4());
    
    Ok(parent.join(temp_name))
}

/// Write data to a temporary file with proper error handling
fn write_temp_file(path: &Path, data: &str) -> Result<()> {
    let mut file = fs::File::create(path)
        .with_context(|| format!("Failed to create temporary file: {:?}", path))?;
    
    file.write_all(data.as_bytes())
        .with_context(|| format!("Failed to write to temporary file: {:?}", path))?;
    
    file.sync_all()
        .with_context(|| format!("Failed to sync temporary file: {:?}", path))?;
    
    Ok(())
}

/// Perform an atomic rename operation
fn atomic_rename(from: &Path, to: &Path) -> Result<()> {
    fs::rename(from, to)
        .with_context(|| format!("Failed to rename {:?} to {:?}", from, to))?;
    
    Ok(())
}

/// Read data from a file with proper error handling
pub fn atomic_read<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();
    
    fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {:?}", path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_atomic_write_success() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.json");
        
        let data = r#"{"test": "data"}"#;
        atomic_write(&file_path, data)?;
        
        let read_data = fs::read_to_string(&file_path)?;
        assert_eq!(read_data, data);
        
        Ok(())
    }
    
    #[test]
    fn test_atomic_write_creates_parent_dirs() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("subdir").join("test.json");
        
        let data = r#"{"test": "data"}"#;
        atomic_write(&file_path, data)?;
        
        assert!(file_path.exists());
        let read_data = fs::read_to_string(&file_path)?;
        assert_eq!(read_data, data);
        
        Ok(())
    }
    
    #[test]
    fn test_atomic_write_overwrites_existing() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.json");
        
        // Write initial data
        atomic_write(&file_path, "initial")?;
        
        // Overwrite with new data
        atomic_write(&file_path, "updated")?;
        
        let read_data = fs::read_to_string(&file_path)?;
        assert_eq!(read_data, "updated");
        
        Ok(())
    }
    
    #[test]
    fn test_no_temp_files_left_on_success() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.json");
        
        atomic_write(&file_path, "data")?;
        
        // Check that no .tmp files exist
        let entries: Vec<_> = fs::read_dir(temp_dir.path())?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "tmp"))
            .collect();
        
        assert!(entries.is_empty(), "Temporary files were not cleaned up");
        
        Ok(())
    }
}