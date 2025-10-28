#!/bin/bash
# Development startup script

# Set environment variables
export ENVIRONMENT=local
export JOB_MATCHING_API_URL=http://127.0.0.1:5555
export RUST_LOG=info,cvenom=debug

# Start the web server
cargo run
