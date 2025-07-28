# Sprint 2: Bug Fixes & Architecture Improvements (Revised - 2 Weeks)

## Overview
Sprint 2 has been revised to prioritize critical bug fixes discovered during testing, particularly the Kanban task creation deadlock. The sprint now balances urgent bug fixes with essential architecture improvements.

## Sprint Status
- **Started**: Sprint 2 Week 1
- **Critical Bug Fixed**: ✅ Kanban task creation deadlock resolved
- **Tests Added**: ✅ Deadlock regression tests implemented

## Week 1: Critical Bug Fixes & Stability (Current Week)

### Completed ✅
1. **CRITICAL: Fixed Kanban Task Creation Deadlock**
   - Root cause: Nested mutex acquisition in `handle_task_form_keys`
   - Solution: Proper lock scoping with RAII blocks
   - Added comprehensive deadlock tests

2. **Fixed Multiple Mutex Issues in Kanban Operations**
   - Refactored `delete_selected_task` to minimize lock time
   - Fixed `move_task_to_status` lock patterns
   - Eliminated nested locking antipatterns

### In Progress / Remaining Week 1 Tasks

#### Systems Architect Tasks
**1. Fix Remaining Mutex Issues** (Priority: HIGH)
- Audit all mutex usage patterns in codebase
- Replace `.unwrap()` with safe `lock_mutex()` in UI modules
- Implement consistent lock/unlock patterns
- Files to review:
  - `src/ui/*.rs` - Multiple `.unwrap()` on mutex locks
  - `src/state/*.rs` - Check for nested locks

**2. Move I/O Operations Outside Mutex Locks** (Priority: HIGH)
- `save_to_disk()` currently called while holding locks
- Refactor to:
  ```rust
  let data = {
      let state = lock_mutex(&self.state)?;
      state.serialize()?
  }; // Lock released
  fs::write(path, data)?; // I/O without lock
  ```
- Affects: kanban_state, notes_state, timer_state

#### Junior Engineer Tasks
**3. Implement Atomic Save Operations** (Priority: HIGH)
- Save to temporary file first
- Rename on successful write
- Rollback on failure
- Prevent data corruption during crashes

**4. UI Mutex Safety Audit** (Priority: MEDIUM)
- Replace all `.unwrap()` with `lock_mutex()` wrapper
- Add proper error handling in render functions
- Files: All `src/ui/*.rs`

#### Project Manager Tasks
**5. Comprehensive Bug Testing** (Priority: HIGH)
- Test all user workflows systematically
- Document any new bugs found
- Create test cases for critical paths
- Update bug tracking documentation

## Week 2: Performance & Architecture

### Systems Architect Tasks

**6. Implement Async I/O Operations** (Priority: HIGH)
- Convert file operations to async with `tokio::fs`
- Add progress indicators for long operations
- Implement debounced saving (max 1 save/second)
- Ensure UI remains responsive

**7. Message-Passing Architecture** (Priority: MEDIUM)
- Design channel-based state management
- Replace direct mutex access where appropriate
- Implement command/event pattern
- Reduce deadlock risks

**8. Service Layer Abstraction** (Priority: MEDIUM)
- Create `src/services/` directory
- Abstract storage operations
- Implement dependency injection
- Prepare for future extensibility

### Junior Engineer Tasks

**9. Configuration Management** (Priority: MEDIUM)
- TOML-based configuration system
- User-customizable settings:
  - Timer durations
  - Key bindings
  - Theme colors
  - Auto-save intervals
- Hot-reload support

**10. Performance Optimizations** (Priority: MEDIUM)
- Migrate read-heavy operations to `RwLock`
- Implement differential rendering
- Add viewport virtualization for large lists
- Profile and benchmark performance

### Project Manager Tasks

**11. Test Coverage to 80%** (Priority: HIGH)
- Expand test suite from 60% to 80%
- Add integration tests for async operations
- Test error recovery scenarios
- Performance regression tests

**12. Data Backup/Restore** (Priority: MEDIUM)
- Versioned backup system
- Automatic daily backups
- Import/export functionality
- Data migration framework

## Bug Tracking

### Fixed Bugs ✅
1. **Kanban Task Creation Deadlock** - Fixed with proper lock scoping
2. **Task Deletion Multiple Lock Acquisitions** - Refactored for efficiency
3. **Task Status Update Lock Pattern** - Eliminated nested locks

### Known Bugs to Fix 🐛
1. **UI Mutex Unwraps** - ~50 instances of unsafe `.unwrap()` on locks
2. **I/O During Lock Hold** - Performance issue, blocks other threads
3. **No Atomic Saves** - Risk of data corruption on crash
4. **Missing Error Recovery** - App doesn't handle corrupted JSON files gracefully

### Potential Issues to Investigate 🔍
1. Timer thread synchronization
2. Concurrent file access from multiple instances
3. Memory usage with large datasets
4. Terminal resize during operations

## Success Metrics

### Week 1 (Bug Fixes)
- [ ] Zero deadlocks in normal operation
- [ ] All mutex operations use safe wrappers
- [ ] I/O operations don't block UI
- [ ] Atomic save operations implemented
- [ ] All critical user flows tested

### Week 2 (Architecture)
- [ ] Async I/O fully implemented
- [ ] Configuration system functional
- [ ] 80% test coverage achieved
- [ ] Performance benchmarks established
- [ ] Backup/restore system working

## Testing Checklist

### Deadlock Tests ✅
- [x] Create task → No freeze
- [x] Delete task → No freeze
- [x] Move task → No freeze
- [x] Concurrent operations → No freeze

### Remaining Tests
- [ ] Save during timer tick → No freeze
- [ ] Large dataset operations → Responsive
- [ ] Crash during save → Data recoverable
- [ ] Corrupted JSON → Graceful recovery
- [ ] Multiple instances → No corruption

## Development Guidelines

### Mutex Best Practices
1. **Always scope locks minimally**
   ```rust
   {
       let guard = lock_mutex(&state)?;
       // minimal operations
   } // lock released
   ```

2. **Never call functions while holding locks**
   ```rust
   // BAD
   let guard = lock_mutex(&state)?;
   other_function()?; // May need same lock!
   
   // GOOD
   let data = {
       let guard = lock_mutex(&state)?;
       guard.get_data()
   };
   other_function(data)?;
   ```

3. **Move I/O outside locks**
   ```rust
   // Serialize inside lock, write outside
   let json = {
       let state = lock_mutex(&state)?;
       serde_json::to_string(&*state)?
   };
   fs::write(path, json)?;
   ```

## Daily Standup Focus

### Week 1 Remaining Days
- Fix remaining mutex issues
- Implement atomic saves
- Test all workflows
- Document findings

### Week 2 Plan
- Day 1-2: Async I/O implementation
- Day 3: Configuration system
- Day 4: Service layer & backup system
- Day 5: Testing & coverage improvement

## Risks & Mitigation

### Technical Risks
1. **More Deadlocks Found**
   - Mitigation: Systematic mutex audit
   - Add timeout-based deadlock detection

2. **Performance Regression**
   - Mitigation: Benchmark before/after changes
   - Profile critical paths

3. **Breaking Changes**
   - Mitigation: Comprehensive test coverage
   - Careful refactoring with tests

## Next Steps

1. Complete remaining Week 1 bug fixes
2. Begin async I/O implementation
3. Continue expanding test coverage
4. Document all architectural decisions

This revised sprint balances critical bug fixes with necessary architectural improvements, ensuring both stability and progress toward a more robust application.