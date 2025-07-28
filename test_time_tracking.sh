#!/bin/bash

# Test script to verify time tracking functionality
echo "=== Time Tracking Test Script ==="
echo ""
echo "This script will help test if time tracking is working correctly."
echo "It will run the app with debug logging enabled."
echo ""
echo "Test Instructions:"
echo "1. Create or select a task in the Kanban board"
echo "2. Press SPACE to start tracking the task"
echo "3. Let the timer run for at least 10 seconds" 
echo "4. Press 'k' to skip/complete the timer"
echo "5. Check the logs below to see if time was added"
echo "6. Press 'q' or ESC to quit"
echo ""
echo "Starting TimberTask with debug logging..."
echo ""

# Enable debug logging for relevant modules
export RUST_LOG=timber_task::app=debug,timber_task::app::kanban=debug,timber_task::state::timer_state=info,timber_task::state::kanban_state=info

# Run the app
cargo run