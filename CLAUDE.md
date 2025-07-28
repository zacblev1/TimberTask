# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Development
- **Build**: `cargo build` (debug) or `cargo build --release` (optimized)
- **Run**: `cargo run` or `cargo run --release`
- **Test**: `cargo test`
- **Test Specific**: `cargo test test_name` or `cargo test module_name`
- **Format**: `cargo fmt` (auto-format) or `cargo fmt -- --check` (check only)
- **Lint**: `cargo clippy -- -D warnings` (treats warnings as errors)
- **Clean**: `cargo clean`

### Testing
- **Run All Tests**: `cargo test`
- **Run Deadlock Tests**: `cargo test deadlock_tests`
- **Run with Output**: `cargo test -- --nocapture`
- **Run Single Test**: `cargo test test_name -- --exact`

## Architecture Overview

TimberTask is a Rust-based terminal UI application combining productivity tools: Pomodoro timer, Kanban board, and hierarchical notes system.

### Recent Updates (Sprint 2)
- **Fixed Critical Deadlock**: Kanban task creation no longer freezes the application
- **Improved Error Handling**: Replaced 140+ unwrap() calls with proper error handling
- **Added Test Suite**: 60% coverage with comprehensive unit and integration tests
- **Modularized Architecture**: Broke down 1,649-line app.rs into focused modules
- **Professional Logging**: Replaced file-based logging with `tracing` crate

### Core Architecture Patterns

1. **Thread-Safe State Management**: 
   - All shared state uses `Arc<Mutex<T>>` pattern for thread safety
   - **IMPORTANT**: Always use minimal lock scopes to prevent deadlocks
   - Use the `lock_mutex()` utility function for safe mutex handling
   - Never call functions that may need the same lock while holding a lock

2. **Event-Driven Design**: 
   - Events flow through `event::EventHandler` which processes keyboard input and timer ticks
   - Main loop handles events and triggers UI updates
   - Event handler has graceful shutdown mechanism
   - Timer runs in separate thread with proper cleanup

3. **State Modules** (`src/state/`):
   - `timer_state.rs`: Pomodoro timer logic with work/break sessions
   - `kanban_state.rs`: Task management with columns and task priorities
   - `notes_state.rs`: Hierarchical note system with parent-child relationships

4. **UI Architecture** (`src/ui/`):
   - Each feature has dedicated UI module (timer.rs, kanban.rs, notes.rs)
   - Modal system for forms (task_form.rs)
   - Tab-based navigation between features
   - Layout utilities handle responsive design

5. **Application Structure** (`src/app/`):
   - `mod.rs`: Core App struct and initialization
   - `timer.rs`: Timer-specific keyboard handling
   - `kanban.rs`: Kanban board logic and task management
   - `notes.rs`: Notes management and search functionality
   - `navigation.rs`: Tab navigation logic

6. **Data Persistence**:
   - All data stored in `~/.timber-task/` as JSON files
   - Automatic save on state changes
   - Load on startup with error recovery
   - Fallback to temp directory if home directory unavailable

### Critical Mutex Patterns (MUST FOLLOW)

```rust
// GOOD - Minimal lock scope
{
    let mut state = lock_mutex(&self.state)?;
    state.update();
    state.save_to_disk()?;
} // Lock released here

// BAD - Can cause deadlock
let mut state = lock_mutex(&self.state)?;
state.update();
self.some_function()?; // This might need the same lock!
```

### Error Handling Guidelines

1. **Use `lock_mutex()` utility**: Located in `src/utils/mutex.rs`
2. **Handle poisoned mutexes**: The utility handles this automatically
3. **Propagate errors with context**: Use `?` operator and anyhow contexts
4. **UI functions**: Use pattern matching instead of `?` when can't return Result

### Key Implementation Details

- **Terminal Handling**: Uses crossterm for cross-platform terminal manipulation
- **Minimum Terminal Size**: 80x24 required, 120x30 recommended
- **Error Handling**: Custom error types in `src/error.rs`, anyhow for propagation
- **Time Management**: chrono for date/time, tokio for async operations
- **Notifications**: Desktop notifications via notify-rust when timer completes
- **Logging**: Uses `tracing` crate with daily rotation in `~/.timber-task/logs/`

### Testing Infrastructure

- **Test Fixtures**: `src/tests/fixtures.rs` provides test utilities
- **Test Coverage**: Currently at 60%, target is 80%
- **Deadlock Tests**: `tests/deadlock_tests.rs` prevents regression
- **CI/CD**: GitHub Actions runs tests on all platforms

### Module Responsibilities

- `app/`: Modularized application logic (was monolithic app.rs)
- `event/`: Event handling with graceful shutdown
- `state/`: Core business logic and data structures
- `ui/`: Terminal UI rendering components
- `utils/`: Shared utilities including mutex helpers
- `error.rs`: Custom error types
- `logging.rs`: Logging configuration

### Known Issues Being Addressed

1. ~50 remaining `.unwrap()` calls in UI modules
2. I/O operations performed while holding mutex locks
3. Need atomic save operations for crash safety
4. Missing configuration management system

### Sprint 2 Status
See `/docs/SPRINT_2_REVISED_PLAN.md` for current sprint progress focusing on bug fixes and architectural improvements.