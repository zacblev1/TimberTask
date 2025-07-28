# TimberTask Error Handling Guidelines

This document outlines the error handling patterns and best practices for the TimberTask project. These guidelines are based on the patterns established during Sprint 1 and should be followed for all future development.

## Core Principles

1. **User Experience First**: Errors should be handled gracefully without crashing the application
2. **Meaningful Messages**: Error messages should be clear and actionable for users
3. **Comprehensive Logging**: All errors should be logged for debugging purposes
4. **Fail Safe**: When in doubt, preserve user data and maintain application state

## Error Types and When to Use Them

### 1. Result vs Option

**Use `Result<T, E>` when:**
- An operation can fail in ways that need to be communicated
- I/O operations (file system, network)
- Data parsing or serialization
- Operations that interact with external systems
- Any operation where the caller needs to know why it failed

```rust
// Good: Clear error context
pub fn save_to_disk(&self) -> Result<()> {
    let data = serde_json::to_string_pretty(&self)
        .map_err(|e| anyhow!("Failed to serialize state: {}", e))?;
    // ...
}
```

**Use `Option<T>` when:**
- The absence of a value is a normal, expected condition
- Looking up items in collections
- Optional configuration values
- When "not found" is not an error condition

```rust
// Good: Looking up an item that may not exist
pub fn get_task(&self, id: &str) -> Option<&Task> {
    self.tasks.get(id)
}
```

### 2. Error Type Hierarchy

The project uses a two-tier error system:

1. **Custom Error Types (`AppError`)**: For domain-specific errors that need special handling
2. **Generic Errors (`anyhow::Error`)**: For general application errors with context

```rust
// Custom errors in src/error.rs
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Mutex lock poisoned: {0}")]
    MutexPoisoned(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    // ... other specific errors
}

// Generic errors with context
return Err(anyhow!("Terminal too small, min 80x24 required, current {}x{}", 
    size.width, size.height));
```

## Mutex Handling

### Pattern for Safe Mutex Access

Always use the utility function for mutex operations to handle poisoning gracefully:

```rust
use crate::utils::mutex::lock_mutex;

// Good: Using the safe wrapper
let mut state = lock_mutex(&self.timer_state)?;

// Avoid: Direct lock() calls
let mut state = self.timer_state.lock().unwrap(); // Don't do this!
```

### Mutex Poisoning Recovery

When a mutex is poisoned, the application should:
1. Log the error with full context
2. Attempt to recover the data if possible
3. Fall back to a safe default state
4. Continue operation without crashing

## Error Propagation Strategies

### 1. Early Returns with Context

Use the `?` operator with context via `map_err`:

```rust
pub fn load_from_disk(&mut self) -> Result<()> {
    let content = fs::read_to_string(&path)
        .map_err(|e| anyhow!("Failed to read state from disk: {}", e))?;
    
    let loaded: Self = serde_json::from_str(&content)
        .map_err(|e| anyhow!("Failed to deserialize state: {}", e))?;
    
    *self = loaded;
    Ok(())
}
```

### 2. Graceful Degradation

For non-critical operations, log and continue:

```rust
// Loading saved data - if it fails, start fresh
if let Err(e) = kanban_state.load_from_disk() {
    warn!("Failed to load kanban data: {}", e);
    // Continue with default state
} else {
    info!("Successfully loaded kanban data");
}
```

### 3. Critical vs Non-Critical Errors

**Critical errors** (must stop execution):
- Terminal initialization failures
- Panic handler setup failures
- Core system resource unavailability

**Non-critical errors** (log and continue):
- Failed to load saved state (use defaults)
- Failed to save state (retry later)
- UI rendering issues (skip frame)

## User-Facing Error Messages

### Guidelines for Error Messages

1. **Be Specific**: Tell the user what went wrong
2. **Be Actionable**: Suggest what they can do about it
3. **Be Concise**: Don't overwhelm with technical details
4. **Be Friendly**: Maintain a helpful tone

```rust
// Good: Clear and actionable
return Err(anyhow!("Terminal too small. Please resize to at least 80x24 characters."));

// Avoid: Technical and vague
return Err(anyhow!("Constraint violation in render bounds calculation"));
```

### Error Display Patterns

For terminal UI applications:
- Critical errors: Display in a modal or status bar
- Warnings: Show temporarily in status area
- Info: Log only, don't interrupt user flow

## Logging Error Conditions

### Logging Levels

Use appropriate log levels for different scenarios:

```rust
// Error: Something failed that shouldn't have
error!("Failed to save user data: {:?}", e);

// Warn: Something failed but we recovered
warn!("Failed to load config, using defaults: {}", e);

// Info: Normal operational messages
info!("Successfully saved state to disk");

// Debug: Detailed information for debugging
debug!("Attempting to lock mutex for timer state");
```

### What to Log

Always include:
1. What operation was being attempted
2. What went wrong
3. Any relevant context (file paths, IDs, etc.)
4. The full error chain

```rust
error!("Failed to save task {} to project {}: {:?}", 
    task_id, project_id, e);
```

## Recovery Strategies

### 1. Automatic Retry

For transient failures (file locks, temporary I/O issues):

```rust
let mut attempts = 0;
loop {
    match save_to_disk() {
        Ok(_) => break,
        Err(e) if attempts < 3 => {
            warn!("Save failed (attempt {}): {}", attempts + 1, e);
            attempts += 1;
            std::thread::sleep(Duration::from_millis(100));
        }
        Err(e) => {
            error!("Failed to save after {} attempts: {}", attempts, e);
            return Err(e);
        }
    }
}
```

### 2. Fallback to Safe State

When loading fails, use safe defaults:

```rust
pub fn load_or_default() -> Self {
    match Self::load_from_disk() {
        Ok(state) => state,
        Err(e) => {
            warn!("Failed to load state, using default: {}", e);
            Self::default()
        }
    }
}
```

### 3. Partial Recovery

Save what can be saved:

```rust
// If one component fails to save, try to save others
let mut errors = Vec::new();

if let Err(e) = self.save_tasks() {
    errors.push(format!("tasks: {}", e));
}

if let Err(e) = self.save_projects() {
    errors.push(format!("projects: {}", e));
}

if !errors.is_empty() {
    error!("Partial save failure: {}", errors.join(", "));
}
```

## Testing Error Conditions

### Test Patterns

Always test both success and failure paths:

```rust
#[test]
fn test_save_with_invalid_path() {
    let state = KanbanState::default();
    // Force an error by using an invalid path
    let result = state.save_to_path("/root/cannot_write_here");
    assert!(result.is_err());
}

#[test]
fn test_load_from_missing_file() {
    let mut state = NotesState::default();
    // Should not panic, should handle gracefully
    let result = state.load_from_disk();
    // Missing file is OK, returns error but doesn't crash
    assert!(result.is_err());
}
```

## Common Patterns and Anti-Patterns

### DO:
- ✅ Add context to errors with `map_err` or `anyhow!`
- ✅ Log errors before propagating them in critical paths
- ✅ Use `unwrap_or_default()` for optional values with sensible defaults
- ✅ Handle mutex poisoning explicitly
- ✅ Test error conditions
- ✅ Provide user-friendly error messages

### DON'T:
- ❌ Use `unwrap()` in production code (except in tests)
- ❌ Use `expect()` without a clear message about why it's safe
- ❌ Silently swallow errors without logging
- ❌ Panic on recoverable errors
- ❌ Expose internal implementation details in user-facing messages
- ❌ Ignore mutex poisoning

## Integration with Application Architecture

### Event Loop Error Handling

The main event loop should never panic:

```rust
loop {
    match event_handler.next()? {
        Event::Key(key) => {
            if let Err(e) = app.handle_key(key) {
                error!("Key handling error: {:?}", e);
                // Continue running - don't crash on input errors
            }
        }
        // ...
    }
}
```

### State Persistence

Save operations should be resilient:

```rust
impl TimerState {
    pub fn complete_session(&mut self) -> Result<()> {
        // Update state first
        self.completed_sessions += 1;
        
        // Then try to persist - if it fails, state is still consistent
        if let Err(e) = self.save_to_disk() {
            error!("Failed to persist timer state: {}", e);
            // State is still updated in memory
        }
        
        Ok(())
    }
}
```

## Summary

These guidelines ensure TimberTask remains stable and user-friendly even when errors occur. The key is to:

1. Use appropriate error types (`Result` vs `Option`)
2. Always handle mutex poisoning through the utility wrapper
3. Add meaningful context to errors
4. Log comprehensively for debugging
5. Recover gracefully without losing user data
6. Provide clear, actionable error messages to users

Following these patterns will maintain consistency across the codebase and provide a robust user experience.