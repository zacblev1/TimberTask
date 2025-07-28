#!/bin/bash

echo "=== Timer Diagnostics ==="
echo

# Set up test data
rm -rf ~/.timber-task
mkdir -p ~/.timber-task

# Create kanban data
cat > ~/.timber-task/kanban_data.json << 'EOF'
{
  "projects": {"1": {"id": "1", "name": "Test", "created_at": 1234567890, "updated_at": 1234567890}},
  "tasks": {
    "1": {
      "id": "1", 
      "project_id": "1", 
      "title": "Test Task", 
      "description": "Test", 
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

echo "Test 1: Check if timer can complete naturally"
echo "Starting app with:"
echo "- Task ID 1 with 0 time_spent"
echo "- Timer will be started manually"
echo

# Create a expect script to automate the test
cat > timer_test.exp << 'EOF'
#!/usr/bin/expect -f
set timeout 20
spawn env DEBUG_TIMER=1 RUST_LOG=timber_task=debug cargo run --release

# Wait for app to start
sleep 1

# Go to kanban
send "\t"
sleep 0.5

# Select the task (should already be selected)
send " "
sleep 0.5

# Go back to timer
send "\t\t"
sleep 0.5

# Start the timer
send "s"
sleep 0.5

# Wait for timer to complete (10 seconds + buffer)
sleep 12

# Check kanban to see if time was added
send "\t"
sleep 1

# Quit
send "q"
send "\003"

expect eof
EOF

chmod +x timer_test.exp

echo "Running automated test..."
./timer_test.exp 2>&1 | grep -E "(Timer|timer|completed|Adding|time_spent|Starting tracking)"

echo
echo "Checking final results..."
if [ -f ~/.timber-task/kanban_data.json ]; then
    echo "Task data after test:"
    cat ~/.timber-task/kanban_data.json | jq '.tasks."1" | {title, time_spent}'
fi

# Cleanup
rm timer_test.exp