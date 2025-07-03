# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Important Notes for Claude Code

* Always commit as current user, not as `claude`.

## Project Overview

YABE (YAml Base Extractor) is a GitOps YAML organizer tool written in Rust. It computes common base configurations among multiple YAML files and generates differences for each file, reducing duplication in GitOps workflows. The tool supports quorum-based diffing, YAML sorting, and both configuration file and command-line interfaces.

## Common Development Commands

### Building and Testing
```bash
# Build the project
cargo build

# Run tests
cargo test

# Build and install locally
cargo install --path .

# Run with debug logging
cargo run -- --debug [other-args]
```

### Running the Tool
```bash
# Basic usage
cargo run -- file1.yaml file2.yaml file3.yaml

# With configuration file
cargo run -- --config config.yaml

# In-place modification
cargo run -- -i -r helm_values.yaml file1.yaml file2.yaml

# Using path patterns
cargo run -- -p "*.yaml" -p "configs/*.yaml"

# Using exclude patterns to skip files
cargo run -- -p "**/*.yaml" --exclude "target" --exclude "*.tmp"

# Sort-only mode with exclusions
cargo run -- --sort-only --sort-config-path ./sort-config.yaml -p "**/*.yaml" --exclude "target"
```

## Architecture

### Core Modules
- **`main.rs`**: CLI interface, argument parsing, and main workflow orchestration
- **`lib.rs`**: Library entry point exposing core functionality  
- **`diff.rs`**: Core diffing logic including `compute_diff()` and `diff_and_common_multiple()`
- **`merge.rs`**: YAML merging functionality for combining base files with overrides
- **`sorter.rs`**: YAML content sorting based on user-defined configurations
- **`deep_equal.rs`**: Deep comparison utility for YAML values

### Key Data Flow
1. **Input Processing**: Parse CLI args and config files, expand glob patterns
2. **File Filtering**: Apply exclude patterns to filter out unwanted files using multiple matching strategies
3. **YAML Loading**: Read and parse all remaining input YAML files
4. **Base Merging**: If existing base provided, merge with each input file
5. **Diff Computation**: Compute diffs against read-only base (if provided)
6. **Quorum Processing**: Extract common base from diffs using quorum percentage
7. **Output Generation**: Write base file and per-file diffs (in-place or to output folder)

### Configuration System
The tool supports both command-line arguments and YAML configuration files. Config file values are overridden by command-line arguments. Key configuration options:
- `read_only_base`: Reference YAML for diff computation
- `base`: Existing base to merge with inputs  
- `quorum`: Percentage threshold for common base extraction
- `sort_config_path`: YAML sorting rules configuration
- `exclude_patterns`: Patterns to exclude files during processing (supports substring, glob, and path component matching)

### Testing Structure
Tests are organized by module:
- `test_diff.rs`: Tests for diff computation and multi-file diffing
- `test_deep_equal.rs`: Tests for YAML comparison utility
- `test_sorter.rs`: Tests for YAML sorting functionality
- `test_common.rs`: Shared test utilities and common test cases