use std::sync::{Mutex, MutexGuard};
use anyhow::{anyhow, Result};

/// Safely lock a mutex, converting poisoned errors to anyhow errors
pub fn lock_mutex<T>(mutex: &Mutex<T>) -> Result<MutexGuard<T>> {
    mutex.lock()
        .map_err(|e| anyhow!("Mutex lock poisoned: {}", e))
}