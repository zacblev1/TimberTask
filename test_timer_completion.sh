#!/bin/bash

echo "=== Testing Timer Completion and Time Tracking ==="
echo

# Clean data
rm -rf ~/.timber-task
mkdir -p ~/.timber-task

# Create initial test data
cat > ~/.timber-task/kanban_data.json << EOF
{
  "projects": {
    "1": {
      "id": "1",
      "name": "Test Project",
      "created_at": 1234567890,
      "updated_at": 1234567890
    }
  },
  "tasks": {
    "1": {
      "id": "1",
      "project_id": "1",
      "title": "Test Task",
      "description": "Testing time tracking",
      "status": "InProgress",
      "priority": "Medium",
      "time_spent": 0,
      "created_at": 1234567890,
      "updated_at": 1234567890
    }
  },
  "next_project_id": 2,
  "next_task_id": 2,
  "selected_project_id": "1"
}
EOF

# Create timer state with task selected and almost complete
cat > ~/.timber-task/timer_state.json << EOF
{
  "start_timestamp": null,
  "remaining_seconds": 2,
  "is_running": false,
  "is_work_period": true,
  "work_seconds": 10,
  "break_seconds": 5,
  "completed_pomodoros": 0,
  "current_task_id": "1"
}
EOF

echo "Initial task time_spent:"
cat ~/.timber-task/kanban_data.json | jq '.tasks."1".time_spent'

echo
echo "Starting TimberTask with 2-second timer..."
echo "The timer should complete quickly and add 10 seconds to the task"
echo

# Run with debug timer and let it complete
timeout 5 bash -c 'DEBUG_TIMER=1 RUST_LOG=timber_task=debug cargo run --release 2>&1 | grep -E "(Timer|timer|completed|Adding|time)"'

echo
echo "Checking results after timer completion..."
echo

if [ -f ~/.timber-task/kanban_data.json ]; then
    TIME_SPENT=$(cat ~/.timber-task/kanban_data.json | jq '.tasks."1".time_spent')
    echo "Task time_spent after test: $TIME_SPENT seconds"
    
    if [ "$TIME_SPENT" -gt "0" ]; then
        echo "✅ SUCCESS: Time was recorded!"
    else
        echo "❌ FAILURE: Time was not recorded"
    fi
else
    echo "❌ ERROR: kanban_data.json not found"
fi