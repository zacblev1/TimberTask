# Sprint 2 Task Tracker

## Week 1: Bug Fixes & Stability

### ✅ Completed Tasks
- [x] **CRITICAL BUG**: Investigate Kanban task creation deadlock
- [x] Create comprehensive bug list from testing  
- [x] Fix critical Kanban deadlock bug
- [x] Add deadlock regression tests

### 🔄 In Progress Tasks
- [ ] Fix remaining mutex issues in codebase
- [ ] Move I/O operations outside of mutex locks
- [ ] Test all user workflows for additional bugs
- [ ] Implement atomic save operations

### 📋 Week 1 Task Details

#### Fix Remaining Mutex Issues
**Owner**: Systems Architect  
**Priority**: HIGH  
**Status**: Not Started  
**Details**:
- Replace ~50 `.unwrap()` calls on mutex locks in UI modules
- Use safe `lock_mutex()` wrapper consistently
- Add proper error handling for poisoned mutexes

#### Move I/O Outside Mutex Locks  
**Owner**: Systems Architect  
**Priority**: HIGH  
**Status**: Not Started  
**Details**:
- Refactor `save_to_disk()` in all state modules
- Serialize data inside lock, write outside
- Improve application responsiveness

#### Implement Atomic Saves
**Owner**: Junior Engineer  
**Priority**: HIGH  
**Status**: Not Started  
**Details**:
- Write to temp file first
- Rename on success
- Implement rollback on failure

#### Test All Workflows
**Owner**: Project Manager  
**Priority**: HIGH  
**Status**: Not Started  
**Details**:
- Systematic testing of all features
- Document any new bugs
- Create regression test cases

## Week 2: Performance & Architecture

### 📅 Planned Tasks

#### Async I/O Implementation
**Owner**: Systems Architect  
**Priority**: HIGH  
**Status**: Not Started  
**Dependencies**: Week 1 I/O refactoring  
**Details**:
- Migrate to `tokio::fs`
- Add progress indicators
- Implement save debouncing

#### Message-Passing Architecture
**Owner**: Systems Architect  
**Priority**: MEDIUM  
**Status**: Not Started  
**Dependencies**: Mutex fixes complete  
**Details**:
- Design channel-based communication
- Implement command/event pattern
- Reduce mutex usage

#### Configuration Management
**Owner**: Junior Engineer  
**Priority**: MEDIUM  
**Status**: Not Started  
**Details**:
- TOML configuration system
- User settings for timers, keys, themes
- Hot-reload support

#### Data Backup/Restore
**Owner**: Junior Engineer  
**Priority**: MEDIUM  
**Status**: Not Started  
**Details**:
- Versioned backups
- Auto-backup daily
- Import/export features

#### Service Layer Abstraction
**Owner**: Systems Architect  
**Priority**: MEDIUM  
**Status**: Not Started  
**Details**:
- Create services directory
- Abstract storage operations
- Dependency injection

#### 80% Test Coverage
**Owner**: Project Manager  
**Priority**: HIGH  
**Status**: Not Started  
**Current**: 60%  
**Target**: 80%  
**Details**:
- Expand unit tests
- Add integration tests
- Performance tests

## Bug Status

### 🐛 Active Bugs
1. **UI Mutex Unwraps** - ~50 unsafe unwrap() calls
2. **I/O During Lock** - Performance bottleneck
3. **No Atomic Saves** - Data corruption risk
4. **Missing Error Recovery** - Can't handle corrupted files

### ✅ Fixed Bugs  
1. **Kanban Task Creation Deadlock** - Fixed with lock scoping
2. **Task Deletion Lock Issues** - Refactored
3. **Task Status Update Deadlock** - Fixed

### 🔍 To Investigate
1. Timer thread synchronization issues
2. Concurrent file access handling
3. Memory usage with 1000+ tasks
4. Terminal resize edge cases

## Progress Metrics

### Week 1 Progress
- Bugs Fixed: 3/7 (43%)
- Tests Added: 3
- Code Coverage: 60%
- Deadlocks Resolved: 3

### Sprint Progress
- Overall: 25% complete
- Week 1: 50% complete
- Week 2: 0% complete

## Risk Register

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| More deadlocks found | HIGH | MEDIUM | Systematic audit |
| Performance regression | MEDIUM | LOW | Benchmark everything |
| Breaking changes | HIGH | LOW | Comprehensive tests |
| Schedule slip | MEDIUM | MEDIUM | Focus on critical bugs |

## Notes
- Kanban deadlock was critical - good catch!
- Need to prioritize mutex safety over new features
- Consider adding mutex timeout detection
- UI responsiveness is key user experience factor