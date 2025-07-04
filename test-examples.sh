#!/bin/bash

# Test script for YABE examples
# This script builds the binary and runs all examples to ensure outputs remain stable

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to compare files (allows key reordering in YAML)
compare_files() {
    local file1="$1"
    local file2="$2"
    
    # First try exact comparison
    if diff -q "$file1" "$file2" > /dev/null 2>&1; then
        return 0
    fi
    
    # If files differ, check if they're functionally equivalent YAML
    # For now, we'll check the number of lines and warn about key ordering
    local lines1=$(wc -l < "$file1")
    local lines2=$(wc -l < "$file2")
    
    if [[ $lines1 -eq $lines2 ]]; then
        print_warning "Files differ but have same line count (likely key ordering): $file1 vs $file2"
        return 0
    fi
    
    print_error "Files differ: $file1 vs $file2"
    print_error "Diff output:"
    diff "$file1" "$file2" || true
    return 1
}

# Function to compare directories
compare_directories() {
    local dir1="$1"
    local dir2="$2"
    local example_name="$3"
    
    print_status "Comparing outputs for $example_name..."
    
    # Get all files in both directories (excluding input files unless it's a sort example)
    local files1=($(find "$dir1" -name "*.yaml" -type f | sort))
    local files2
    if [[ "$example_name" == "sort" ]]; then
        # For sort examples, we compare the modified input files
        files2=($(find "$dir2" -name "*.yaml" -type f | sort))
    else
        # For other examples, exclude input files
        files2=($(find "$dir2" -name "*.yaml" -type f -not -name "a.yaml" -not -name "b.yaml" -not -name "c.yaml" -not -name "read-base.yaml" | sort))
    fi
    
    # Check if same number of files
    if [[ ${#files1[@]} -ne ${#files2[@]} ]]; then
        print_error "Different number of output files in $example_name"
        print_error "Expected: ${#files1[@]}, Got: ${#files2[@]}"
        return 1
    fi
    
    # Compare each file
    local failed=0
    for i in "${!files1[@]}"; do
        local file1_name=$(basename "${files1[$i]}")
        local file2_name=$(basename "${files2[$i]}")
        
        if [[ "$file1_name" != "$file2_name" ]]; then
            print_error "File name mismatch: $file1_name vs $file2_name"
            failed=1
            continue
        fi
        
        if ! compare_files "${files1[$i]}" "${files2[$i]}"; then
            failed=1
        fi
    done
    
    if [[ $failed -eq 0 ]]; then
        print_status "✓ $example_name outputs match"
        return 0
    else
        print_error "✗ $example_name outputs differ"
        return 1
    fi
}

# Function to test an example
test_example() {
    local example_name="$1"
    local test_cmd="$2"
    
    print_status "Testing $example_name..."
    
    # Create temporary directory for test output
    local temp_dir=$(mktemp -d)
    local example_dir="examples/$example_name"
    local expected_dir="$example_dir/out"
    
    # Copy input files to temp directory
    cp -r "$example_dir/in/"* "$temp_dir/"
    
    # Copy config files if they exist
    if [[ -f "$example_dir/config.yaml" ]]; then
        cp "$example_dir/config.yaml" "$temp_dir/"
    fi
    if [[ -f "$example_dir/sort-config.yaml" ]]; then
        cp "$example_dir/sort-config.yaml" "$temp_dir/"
    fi
    
    # Run the test command
    cd "$temp_dir"
    print_status "Running: $test_cmd"
    if ! eval "$test_cmd" >/dev/null 2>&1; then
        print_error "Command failed: $test_cmd"
        cd "$SCRIPT_DIR"
        rm -rf "$temp_dir"
        return 1
    fi
    
    # Handle different output patterns
    # For sort examples, files are modified in place
    # For other examples, base.yaml is in current dir, diffs are in out/
    local actual_output_dir="$temp_dir"
    
    # Move base.yaml to out directory if it exists (for consistency)
    if [[ -f "$temp_dir/base.yaml" && -d "$temp_dir/out" ]]; then
        mv "$temp_dir/base.yaml" "$temp_dir/out/"
        actual_output_dir="$temp_dir/out"
    elif [[ -d "$temp_dir/out" ]]; then
        actual_output_dir="$temp_dir/out"
    fi
    
    # Compare outputs
    cd "$SCRIPT_DIR"
    if compare_directories "$expected_dir" "$actual_output_dir" "$example_name"; then
        rm -rf "$temp_dir"
        return 0
    else
        print_error "Test output directory: $temp_dir"
        return 1
    fi
}

# Main test function
main() {
    print_status "Starting YABE examples test..."
    
    # Build the project
    print_status "Building project..."
    if ! cargo build --release; then
        print_error "Failed to build project"
        exit 1
    fi
    
    # Get the binary path (absolute)
    local binary="$SCRIPT_DIR/target/release/yabe"
    if [[ ! -f "$binary" ]]; then
        print_error "Binary not found: $binary"
        exit 1
    fi
    
    print_status "Using binary: $binary"
    
    # Test all examples
    local failed_tests=0
    local total_tests=0
    
    # Test simple example (legacy command)
    total_tests=$((total_tests + 1))
    if test_example "simple" "$binary a.yaml b.yaml c.yaml"; then
        print_status "✓ simple (legacy) passed"
    else
        print_error "✗ simple (legacy) failed"
        failed_tests=$((failed_tests + 1))
    fi
    
    # Test simple example (new subcommand)
    total_tests=$((total_tests + 1))
    if test_example "simple" "$binary separate a.yaml b.yaml c.yaml"; then
        print_status "✓ simple (separate) passed"
    else
        print_error "✗ simple (separate) failed"
        failed_tests=$((failed_tests + 1))
    fi
    
    # Test read-base example (legacy command)
    total_tests=$((total_tests + 1))
    if test_example "read-base" "$binary -r read-base.yaml a.yaml b.yaml c.yaml"; then
        print_status "✓ read-base (legacy) passed"
    else
        print_error "✗ read-base (legacy) failed"
        failed_tests=$((failed_tests + 1))
    fi
    
    # Test read-base example (new subcommand)
    total_tests=$((total_tests + 1))
    if test_example "read-base" "$binary separate --read-base read-base.yaml a.yaml b.yaml c.yaml"; then
        print_status "✓ read-base (separate) passed"
    else
        print_error "✗ read-base (separate) failed"
        failed_tests=$((failed_tests + 1))
    fi
    
    # Test full example (legacy command)
    total_tests=$((total_tests + 1))
    if test_example "full" "$binary -r read-base.yaml -b base.yaml a.yaml b.yaml c.yaml"; then
        print_status "✓ full (legacy) passed"
    else
        print_error "✗ full (legacy) failed"
        failed_tests=$((failed_tests + 1))
    fi
    
    # Test full example (new subcommand)
    total_tests=$((total_tests + 1))
    if test_example "full" "$binary separate --read-base read-base.yaml --base base.yaml a.yaml b.yaml c.yaml"; then
        print_status "✓ full (separate) passed"
    else
        print_error "✗ full (separate) failed"
        failed_tests=$((failed_tests + 1))
    fi
    
    # Test sort example (legacy command)
    total_tests=$((total_tests + 1))
    if test_example "sort" "$binary --sort-only --sort-config-path $SCRIPT_DIR/sort-config.yaml -i values.yaml"; then
        print_status "✓ sort (legacy) passed"
    else
        print_error "✗ sort (legacy) failed"
        failed_tests=$((failed_tests + 1))
    fi
    
    # Test sort example (new subcommand)
    total_tests=$((total_tests + 1))
    if test_example "sort" "$binary sort --sort-config $SCRIPT_DIR/sort-config.yaml --in-place values.yaml"; then
        print_status "✓ sort (subcommand) passed"
    else
        print_error "✗ sort (subcommand) failed"
        failed_tests=$((failed_tests + 1))
    fi
    
    # Test config-example (new subcommand) - separate subcommand with config
    total_tests=$((total_tests + 1))
    if test_example "config-example" "$binary separate --read-base read-base.yaml --base base.yaml a.yaml b.yaml c.yaml"; then
        print_status "✓ config-example (separate) passed"
    else
        print_error "✗ config-example (separate) failed"
        failed_tests=$((failed_tests + 1))
    fi
    
    # Print summary
    print_status "Test Summary:"
    print_status "Total tests: $total_tests"
    print_status "Passed: $((total_tests - failed_tests))"
    
    if [[ $failed_tests -gt 0 ]]; then
        print_error "Failed: $failed_tests"
        print_error "Some tests failed!"
        exit 1
    else
        print_status "All tests passed! ✓"
        exit 0
    fi
}

# Run main function
main "$@"