# Timber Task

A terminal-based productivity application built in Rust that combines a Pomodoro timer, Kanban board, and hierarchical notes system.

## Features

### Pomodoro Timer
- 25-minute work periods and 5-minute break periods
- Track elapsed time for tasks
- Start, pause, reset, and toggle between work/break periods
- Automatically logs time to selected tasks

### Kanban Board
- Organize tasks in Todo, In Progress, and Done columns
- Create, edit, move, and delete tasks
- Track time spent on each task
- Select tasks for time tracking with the Pomodoro timer

### Notes System
- Hierarchical notes with parent-child relationships
- Create, edit, and delete notes
- Expand/collapse note trees
- Search notes by content
- Tag notes for better organization

## Controls

### Global
- <Tab> / <Shift+Tab>: Cycle through tabs
- F1: Show help
- F2: Show settings
- Esc: Close dialogs or cancel
- q: Quit

### Timer Tab
- s: Start/skip timer
- p: Pause timer and save time
- r: Reset timer
- t: Toggle between work and break periods

### Kanban Tab
- h/j/k/l or Arrow keys: Navigate tasks
- n: Create new task
- i: Move task to In Progress
- d: Move task to Done
- Space: Select task for time tracking (also moves task to In Progress)
- t: Add time to task manually
- x: Delete task

### Notes Tab
- h/j/k/l or Arrow keys: Navigate notes
- n: Create new note
- Enter: Edit selected note
- e: Toggle note expanded/collapsed
- d: Delete selected note
- c: Create child note
- /: Search notes
- t: Manage tags

## Installation

### Prerequisites
- Rust and Cargo (https://rustup.rs/)

### Building from Source
```bash
# Clone the repository
git clone https://github.com/zacblev1/TimberTask.git
cd TimberTask

# Build the application
cargo build --release

# Run the application
cargo run --release
```

## Usage

When you first start the application, you'll be on the timer tab. Use Tab to cycle through the different features.

### Getting Started
1. In the Kanban tab (Tab key), create a task with 'n'
2. Move the task to "In Progress" with 'i'
3. Press Space to select it for time tracking
4. Switch to the Timer tab and start the timer with 's'
5. Time will be logged to your task automatically

## Data Storage

All data is saved automatically to the following location:
- ~/.timber-task/

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.