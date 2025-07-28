#!/bin/bash

echo "=== TimberTask Time Tracking Debug Script ==="
echo
echo "This script will help debug time tracking issues."
echo "It will show all relevant logs for timer and task operations."
echo

# Create backup of current data
echo "Creating backup of current data..."
cp -r ~/.timber-task ~/.timber-task.backup.$(date +%s) 2>/dev/null || true

echo "Starting TimberTask with debug logging..."
echo "Instructions:"
echo "1. Press Tab to go to Kanban board"
echo "2. Create a task (press 'n')"
echo "3. Select the task and press Space to start tracking"
echo "4. Go to Timer tab (Tab) and wait for timer to complete OR press 'k' to skip"
echo "5. Check if time was added to the task"
echo "6. Press Ctrl+C to exit and see the logs"
echo
echo "Press Enter to continue..."
read

# Run with debug logging, filtering for relevant messages
RUST_LOG=timber_task=debug cargo run 2>&1 | grep -E "(Timer|timer|task|Task|add_time|current_task|elapsed|completed|tracking|SaveRequest)" | tee time_tracking_debug.log

echo
echo "=== Debug Summary ==="
echo "Logs saved to: time_tracking_debug.log"
echo
echo "Key things to check:"
echo "1. Did 'Timer completed!' appear?"
echo "2. Was there a task_id when timer completed?"
echo "3. Did 'Adding X seconds to task Y' appear?"
echo "4. Did 'Task time updated' appear?"
echo
echo "Checking current task data..."
echo

# Show current kanban data
if [ -f ~/.timber-task/kanban_data.json ]; then
    echo "Current tasks with time:"
    cat ~/.timber-task/kanban_data.json | jq '.tasks | to_entries | .[] | select(.value.time_spent > 0) | {id: .key, title: .value.title, time_spent: .value.time_spent}'
fi