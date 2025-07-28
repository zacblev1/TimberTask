# TimberTask Testing Guide

## Overview

TimberTask uses a comprehensive test suite to ensure reliability and maintainability. The test architecture includes unit tests, integration tests, and test utilities designed to cover all critical functionality.

## Test Structure

```
tests/
├── common/
│   └── mod.rs          # Shared test utilities and fixtures
├── timer_state_tests.rs    # Unit tests for timer functionality
├── kanban_state_tests.rs   # Unit tests for kanban board
├── notes_state_tests.rs    # Unit tests for notes system
└── integration_tests.rs    # Cross-module integration tests
```

## Running Tests

### Quick Start
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test timer_state_tests

# Run specific test
cargo test test_timer_start

# Use the test runner script
./scripts/run-tests.sh
```

### Test Categories

#### Unit Tests
Focus on individual modules in isolation:
- **Timer State Tests**: Pomodoro timer logic, state transitions
- **Kanban State Tests**: Task/project CRUD, status updates
- **Notes State Tests**: Hierarchical notes, tags, search

#### Integration Tests
Test interactions between modules:
- Full workflow scenarios
- Data persistence across modules
- Concurrent operations
- Performance with large datasets

### Coverage Goals

- **Week 1**: 60% coverage (current focus)
- **Sprint 2**: 80% coverage target

## Test Utilities

### TestFixture
Provides temporary directories and pre-configured states:
```rust
let fixture = TestFixture::new();
let kanban = fixture.create_kanban_state();
```

### TestFactory
Creates test data with sensible defaults:
```rust
let task = TestFactory::create_task("My Task");
let project = TestFactory::create_project("My Project");
let note = TestFactory::create_note("My Note");
```

### Helper Macros
- `assert_err!`: Assert that an operation returns an error
- `assert_mutex_locked!`: Verify mutex is not locked

## Testing Patterns

### Thread Safety Testing
```rust
let state = Arc::new(Mutex::new(State::new()));
let mut handles = vec![];

for i in 0..10 {
    let state_clone = Arc::clone(&state);
    handles.push(thread::spawn(move || {
        // Concurrent operations
    }));
}
```

### Persistence Testing
```rust
// Save state
state.save_to_disk()?;

// Create new instance and load
let mut new_state = State::new();
new_state.load_from_disk()?;

// Verify data integrity
assert_eq!(new_state.data, expected_data);
```

### Error Handling Testing
```rust
// Test invalid operations
assert!(state.update_non_existent("id").is_err());

// Test edge cases
assert!(state.move_to_invalid_parent("id").is_err());
```

## Best Practices

1. **Isolation**: Each test should be independent
2. **Clarity**: Test names should describe what they test
3. **Coverage**: Test both happy paths and error cases
4. **Performance**: Keep tests fast (< 100ms per test)
5. **Determinism**: Avoid time-dependent or random behavior

## Adding New Tests

1. Identify the module to test
2. Create test file if needed: `tests/module_name_tests.rs`
3. Use TestFixture for temporary files
4. Follow existing patterns for consistency
5. Run tests locally before committing

## Continuous Integration

Tests run automatically on:
- Every push to main
- Every pull request
- Can be triggered manually

## Debugging Tests

```bash
# Run single test with println! output
cargo test test_name -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test

# Run specific test file
cargo test --test timer_state_tests
```

## Coverage Reports

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML report
cargo tarpaulin --out Html

# View report
open tarpaulin-report.html
```

## Known Issues

- Some timing-based tests may occasionally fail on heavily loaded systems
- File system tests require write permissions in temp directory
- Coverage tool may not work on all platforms

## Future Improvements

- [ ] Add property-based testing with proptest
- [ ] Implement benchmark tests for performance regression
- [ ] Add fuzzing for robustness testing
- [ ] Create visual test runner dashboard
- [ ] Add mutation testing