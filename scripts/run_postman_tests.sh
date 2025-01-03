#!/bin/bash

# Install dependencies if needed
npm install

# Start the server in background
cargo run &
SERVER_PID=$!

# Wait for server to start
sleep 5

# Run general tests
echo "Running general tests..."
npx newman run tests/postman/fitness_workout_tracker_api_general.postman_collection.json \
    -e tests/postman/local.postman_environment.json

GENERAL_TEST_RESULT=$?

# Run auth tests
echo "Running auth tests..."
npx newman run tests/postman/fitness_workout_tracker_api_auth.postman_collection.json \
    -e tests/postman/local.postman_environment.json

AUTH_TEST_RESULT=$?

# Run workout tests
echo "Running workout tests..."
npx newman run tests/postman/fitness_workout_tracker_api_workouts.postman_collection.json \
    -e tests/postman/local.postman_environment.json

WORKOUT_TEST_RESULT=$?

# Run exercise tests
echo "Running exercise tests..."
npx newman run tests/postman/fitness_workout_tracker_api_exercises.postman_collection.json \
    -e tests/postman/local.postman_environment.json

EXERCISE_TEST_RESULT=$?

# Kill the server
kill $SERVER_PID

# Exit with error if any test failed
if [ $AUTH_TEST_RESULT -ne 0 ] || [ $WORKOUT_TEST_RESULT -ne 0 ] || [ $EXERCISE_TEST_RESULT -ne 0 ]; then
    exit 1
fi

exit 0 