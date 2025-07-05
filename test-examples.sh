#!/bin/bash

set -eo pipefail

# Build the project
echo "Building project..."
cargo build --release

# Get the binary path
BINARY_PATH="$(pwd)/target/release/yabe"

# Add the binary to PATH for the Makefiles
export PATH="$(dirname "$BINARY_PATH"):$PATH"

echo "Running tests with binary: $BINARY_PATH"

for dir in examples/*/; do
    echo "Running tests in $dir"
    (cd "$dir" && make test)
done

if git status --porcelain | grep -E -q '^ [AM]'; then
    echo "There are new or modified files in the examples directories."
    exit 1
else
    echo "All tests passed and no new or modified files found."
fi