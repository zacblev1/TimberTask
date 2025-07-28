# SaveRequest Pattern Implementation

This document describes the SaveRequest pattern implemented in the kanban_state module to prevent mutex deadlocks by separating state mutations from I/O operations.

## Problem

Previously, methods in `kanban_state.rs` would:
1. Lock the mutex
2. Mutate state
3. Call `save_to_disk()` while still holding the lock
4. Release the lock

This could cause deadlocks if `save_to_disk()` or any called function needed to acquire the same or another mutex.

## Solution

The SaveRequest pattern separates state mutations from I/O operations:

1. Methods that mutate state now return a `SaveRequest` enum
2. The mutex lock is released after mutation
3. The save operation is performed outside the mutex lock

## Implementation

### 1. SaveRequest Enum

```rust
#[derive(Debug, Clone)]
pub enum SaveRequest {
    /// Save the entire kanban state
    Full,
    /// No save needed
    None,
}
```

### 2. Updated Method Signatures

Before:
```rust
pub fn create_project(&mut self, name: &str) -> Result<Project>
pub fn create_task_in_project(&mut self, project_id: &str, title: &str, description: &str) -> Result<Task>
pub fn update_task_status(&mut self, task_id: &str, status: TaskStatus) -> Result<Task>
```

After:
```rust
pub fn create_project(&mut self, name: &str) -> Result<(Project, SaveRequest)>
pub fn create_task_in_project(&mut self, project_id: &str, title: &str, description: &str) -> Result<(Task, SaveRequest)>
pub fn update_task_status(&mut self, task_id: &str, status: TaskStatus) -> Result<(Task, SaveRequest)>
```

### 3. Usage Pattern

```rust
// Create task and get save request
let save_request = {
    let mut kanban = lock_mutex(&self.kanban_state)?;
    
    if let Some(project) = kanban.get_selected_project() {
        let project_id = project.id.clone();
        match kanban.create_task_in_project(&project_id, &title, &description) {
            Ok((_task, save_req)) => Some(save_req),
            Err(_) => None,
        }
    } else {
        None
    }
}; // Lock is dropped here

// Process save request outside of mutex lock
if let Some(save_req) = save_request {
    let kanban = lock_mutex(&self.kanban_state)?;
    kanban.process_save_request(&save_req)?;
}
```

## Benefits

1. **Prevents Deadlocks**: I/O operations never happen while holding mutex locks
2. **Clear Separation**: State mutations are clearly separated from side effects
3. **Flexible**: Can be extended to support different save strategies (e.g., batching, async)
4. **Testable**: State mutations can be tested without I/O

## Future Extensions

The pattern can be extended to support:
- Batch saves (accumulate multiple SaveRequests)
- Async saves
- Different save strategies based on request type
- Optimistic updates with rollback on save failure