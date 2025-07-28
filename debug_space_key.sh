#!/bin/bash

# Script to debug Space key issue in TimberTask

echo "Debug Space Key Issue in TimberTask"
echo "==================================="
echo ""
echo "Instructions:"
echo "1. Run TimberTask with this command: RUST_LOG=timber_task=debug cargo run"
echo "2. Press Tab to switch to the Kanban board"
echo "3. Create a task and move it to 'In Progress' column"
echo "4. Select the task (it should be highlighted)"
echo "5. Press Space to try tracking the task"
echo "6. Check the debug output to see what happens"
echo ""
echo "Expected debug output when pressing Space:"
echo "- 'Space key pressed in kanban view'"
echo "- 'toggle_task_tracking called'"
echo "- 'Selected task at column: 1, row: X'"
echo "- 'get_selected_task_id called'"
echo "- 'Found task: id=XXX, title=YYY'"
echo "- 'Starting to track task: XXX'"
echo ""
echo "Running TimberTask with debug logging..."
echo ""

# Run TimberTask with debug logging
RUST_LOG=timber_task=debug cargo run