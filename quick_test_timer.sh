#!/bin/bash

echo "=== Quick Timer Test with 10-second work periods ==="
echo
echo "This will test time tracking with very short timers (10 seconds work, 5 seconds break)"
echo

# Clean up old data for a fresh test
echo "Backing up and cleaning data for fresh test..."
mv ~/.timber-task ~/.timber-task.backup.$(date +%s) 2>/dev/null || true
mkdir -p ~/.timber-task

echo "Starting TimberTask with DEBUG_TIMER mode (10 second work periods)..."
echo
echo "Quick Test Steps:"
echo "1. Press Tab to go to Kanban"
echo "2. Press 'n' to create a task named 'Test Task'"
echo "3. Press Space to start tracking (task auto-moves to In Progress)"
echo "4. Press Tab to go back to Timer tab"
echo "5. Wait 10 seconds for timer to complete (or press 'k' to skip)"
echo "6. Go back to Kanban (Tab) and check if task shows time"
echo
echo "Press Enter to start..."
read

# Run with debug timer (10 second periods) and debug logging
DEBUG_TIMER=1 RUST_LOG=timber_task=debug cargo run 2>&1 | tee full_debug.log | grep -E "(Timer|timer|task|Task|add_time|current_task|elapsed|completed|tracking|SaveRequest|Adding.*seconds)"

echo
echo "=== Test Complete ==="
echo
echo "Checking results..."

# Check if any tasks have time recorded
if [ -f ~/.timber-task/kanban_data.json ]; then
    echo
    echo "Tasks with recorded time:"
    cat ~/.timber-task/kanban_data.json | jq '.tasks | to_entries | .[] | select(.value.time_spent > 0) | {id: .key, title: .value.title, time_spent: .value.time_spent, status: .value.status}'
    
    echo
    echo "All tasks:"
    cat ~/.timber-task/kanban_data.json | jq '.tasks | to_entries | .[] | {id: .key, title: .value.title, time_spent: .value.time_spent, status: .value.status}'
fi

echo
echo "Full logs saved to: full_debug.log"
echo
echo "Key indicators of success:"
echo "- 'Timer completed!' message"
echo "- 'Adding X seconds to task Y' message"  
echo "- Task shows time_spent > 0 in the JSON above"