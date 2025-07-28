#!/bin/bash

echo "=== Monitoring Task Time Tracking ==="
echo
echo "Current task being tracked:"
cat ~/.timber-task/timer_state.json | jq '{task_id: .current_task_id, is_running, remaining_seconds}'

echo
echo "Current task time:"
TASK_ID=$(cat ~/.timber-task/timer_state.json | jq -r '.current_task_id')
cat ~/.timber-task/kanban_data.json | jq ".tasks.\"$TASK_ID\" | {id, title, time_spent}"

echo
echo "Watching for changes... (Press Ctrl+C to stop)"
echo

# Monitor the file for changes
while true; do
    NEW_TIME=$(cat ~/.timber-task/kanban_data.json | jq ".tasks.\"$TASK_ID\".time_spent")
    echo -ne "\rTask $TASK_ID time_spent: $NEW_TIME seconds   "
    sleep 1
done