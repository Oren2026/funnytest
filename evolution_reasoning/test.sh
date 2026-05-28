#!/bin/bash
set -e

echo "=== Evolution Reasoning Tool v0.4 Tests ==="
echo ""

cd "$(dirname "$0")"

# Use full path for cargo
CARGO="/Users/oren/.cargo/bin/cargo"

echo "Building project..."
$CARGO build 2>&1
if [ $? -ne 0 ]; then
    echo "BUILD FAILED"
    exit 1
fi
echo "Build successful!"
echo ""

echo "Running unit tests..."
$CARGO test --lib --tests 2>&1
if [ $? -ne 0 ]; then
    echo "TESTS FAILED"
    exit 1
fi
echo "All tests passed!"
echo ""

echo "=== All tests passed! ==="
exit 0
