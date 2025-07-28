#!/bin/bash

# Test logging functionality

echo "Testing TimberTask logging system..."

# Run the app with debug logging enabled
RUST_LOG=timber_task=debug cargo run &
APP_PID=$!

# Give it a moment to start up
sleep 2

# Kill the app gracefully
kill $APP_PID 2>/dev/null

# Wait for it to finish
wait $APP_PID 2>/dev/null

echo ""
echo "Checking for log files..."

# Check if logs directory was created
if [ -d ~/.timber-task/logs ]; then
    echo "✓ Logs directory created successfully"
    
    # Find today's log file
    LOG_FILE=$(ls -t ~/.timber-task/logs/*.log 2>/dev/null | head -1)
    
    if [ -n "$LOG_FILE" ]; then
        echo "✓ Log file created: $LOG_FILE"
        echo ""
        echo "Last 20 lines of log:"
        echo "===================="
        tail -20 "$LOG_FILE"
    else
        echo "✗ No log files found"
    fi
else
    echo "✗ Logs directory not created"
fi