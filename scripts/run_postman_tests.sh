#!/bin/bash

# Install dependencies if needed
npm install

# Start the server in background
cargo run &
SERVER_PID=$!

# Wait for server to start
sleep 2

# Run tests
npm run test:e2e

# Store the test result
TEST_RESULT=$?

# Kill the server
kill $SERVER_PID

# Exit with the test result
exit $TEST_RESULT 