# Sprint 2: Critical Bug Fixes (2 Weeks)

## Overview
Sprint 2 pivots to focus entirely on fixing critical bugs discovered during testing. The most severe issue is a deadlock when creating Kanban tasks that completely freezes the application.

## Critical Bugs (Week 1)

### 1. **CRITICAL: Kanban Task Creation Deadlock** 🔴
**File**: `src/app/kanban.rs`, lines 108-131
**Issue**: Application freezes when saving a new Kanban task
**Root Cause**: 
- `handle_task_form_keys` locks `kanban_state` at line 112
- While holding the lock, it calls `select_first_available_task()` at line 130
- `select_first_available_task()` tries to acquire the same lock at line 233
- Deadlock occurs because Mutex cannot be locked twice by the same thread

**Fix**:
```rust
// Current problematic code
let kanban = self.kanban_state.clone();
let mut kanban = lock_mutex(&kanban)?;
// ... create task ...
let _ = kanban.save_to_disk();
// Still holding lock here!
self.select_first_available_task()?; // DEADLOCK!

// Fixed code
{
    let kanban = self.kanban_state.clone();
    let mut kanban = lock_mutex(&kanban)?;
    // ... create task ...
    let _ = kanban.save_to_disk();
} // Lock released here
self.select_first_available_task()?; // Safe to call now
```

### 2. **HIGH: Multiple Mutex Lock Acquisitions in delete_selected_task** 🟠
**File**: `src/app/kanban.rs`, lines 285-292
**Issue**: Inefficient multiple lock/unlock cycles
**Details**: The function drops and reacquires locks multiple times, which could lead to race conditions

### 3. **HIGH: Potential Deadlock in move_task_to_status** 🟠
**File**: `src/app/kanban.rs`, lines 308-356
**Issue**: Complex lock/unlock pattern with multiple acquisitions
**Risk**: Similar pattern to the create task bug, could deadlock under certain conditions

### 4. **MEDIUM: File I/O While Holding Mutex** 🟡
**Throughout codebase**
**Issue**: `save_to_disk()` is called while holding mutex locks
**Impact**: Long I/O operations block all other threads from accessing state
**Files affected**:
- `src/state/kanban_state.rs`: lines 321, 344
- `src/state/notes_state.rs`: Check for similar patterns
- `src/state/timer_state.rs`: Check for similar patterns

## Additional Bugs to Investigate (Week 1-2)

### 5. **Error Handling in UI Code**
- Many UI functions still use `.unwrap()` on mutex locks
- Should use the safe `lock_mutex()` wrapper consistently
- Files: All `src/ui/*.rs` files

### 6. **Race Conditions in Timer Thread**
- Timer updates state from a separate thread
- Need to verify thread-safe access patterns
- Check for potential data races

### 7. **Data Loss on Crash**
- If app crashes during save operations, data might be corrupted
- Need atomic save operations (save to temp file, then rename)

## Testing & Validation (Week 2)

### 8. **Comprehensive Deadlock Testing**
- Create tests that stress concurrent operations
- Test rapid task creation/deletion
- Test state changes while timer is running
- Verify no deadlocks in any user flow

### 9. **Error Recovery Testing**
- Test behavior when files are read-only
- Test recovery from corrupted JSON files
- Test handling of missing directories
- Test concurrent access from multiple app instances

### 10. **Performance Under Load**
- Test with 1000+ tasks
- Test with deep note hierarchies (100+ levels)
- Measure UI responsiveness during saves
- Profile mutex contention

## Implementation Plan

### Week 1: Critical Fixes
**Day 1-2**: Fix Kanban task creation deadlock
- Implement proper lock scoping
- Add integration tests for task creation
- Verify fix with manual testing

**Day 3-4**: Fix all mutex-related issues
- Audit all mutex lock patterns
- Implement consistent lock/unlock patterns
- Move I/O operations outside of locks

**Day 5**: Implement atomic save operations
- Save to temporary files first
- Rename on successful write
- Add rollback on failure

### Week 2: Testing & Hardening
**Day 1-2**: Create comprehensive test suite
- Deadlock detection tests
- Concurrent operation tests
- Error injection tests

**Day 3-4**: Fix remaining bugs found during testing
- Address UI mutex issues
- Fix any race conditions
- Improve error messages

**Day 5**: Performance validation
- Run load tests
- Profile and optimize hot paths
- Document performance characteristics

## Success Criteria

1. **No Deadlocks**: Application never freezes during normal operation
2. **Data Integrity**: No data loss even during crashes
3. **Thread Safety**: All concurrent operations are safe
4. **Performance**: UI remains responsive (<100ms) during all operations
5. **Error Recovery**: Graceful handling of all error conditions

## Quick Fixes (Do Immediately)

1. **Add mutex deadlock detection in debug builds**
```rust
#[cfg(debug_assertions)]
use parking_lot::Mutex; // parking_lot has deadlock detection
```

2. **Add operation timeouts**
```rust
match mutex.try_lock_for(Duration::from_secs(5)) {
    Some(guard) => guard,
    None => panic!("Mutex deadlock detected!"),
}
```

3. **Add logging for all mutex operations**
```rust
debug!("Acquiring kanban_state lock in {}", function_name);
let guard = lock_mutex(&self.kanban_state)?;
debug!("Released kanban_state lock in {}", function_name);
```

## Testing Checklist

- [ ] Create new task → No freeze
- [ ] Delete task → No freeze  
- [ ] Move task between columns → No freeze
- [ ] Save while timer running → No freeze
- [ ] Rapid task operations → No freeze
- [ ] Kill app during save → Data recoverable
- [ ] Corrupt JSON file → App starts with error message
- [ ] 1000 tasks → UI still responsive
- [ ] Multiple app instances → No data corruption

## Notes for Developers

1. **Always scope mutex locks to minimum required code**
2. **Never call functions that acquire locks while holding a lock**
3. **Move I/O operations outside of lock scope**
4. **Use RAII pattern - locks drop automatically at scope end**
5. **Test concurrent operations thoroughly**

This sprint focuses entirely on stability and reliability. No new features should be added until all critical bugs are resolved.