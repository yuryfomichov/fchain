#!/bin/bash

# Generate the documentation
cargo doc

# Serve the documentation on port 8000, binding to all interfaces
cd target/doc && python3 -m http.server 8000 --bind 0.0.0.0 