# YAML Diff Tool for Helm Chart Overrides

## Introduction

This project provides a command-line tool written in Rust for computing the differences between multiple YAML files, specifically in the context of Helm chart overrides. The tool helps identify common base configurations and differences among multiple override files while considering a Helm values file as the base layer.

## Features

- **Compute Differences**: Calculates the differences between multiple YAML override files and a Helm values file.
- **Common Base Extraction**: Identifies the common base configuration among the override files.
- **Inplace Modification**: Optionally modifies the original override files to contain only their differences.
- **Debug Logging**: Provides detailed debug logging to understand the processing steps.
- **Recursive Comparison**: Handles complex nested structures in YAML files.

## Installation

### Prerequisites

- **Rust Toolchain**: Ensure you have Rust installed. You can install it from [rustup.rs](https://rustup.rs/).

### Build the Project

Clone the repository and build the project using Cargo:

```bash
git clone https://github.com/dvrkn/yaml-base-extractor.git
cd yaml-base-extractor
cargo build --release
```
This will create an executable in the target/release directory.

## Usage

**Command-Line Arguments**
```bash
Usage: yabe [OPTIONS] [INPUT_FILES]...

Arguments:
  [INPUT_FILES]...  Input YAML files

Options:
  -h, --helm <HELM_VALUES_FILE>  Helm chart values file
  -i, --inplace                  Modify the original input files with diffs
      --debug                    Enable debug logging
  -q, --quorum <QUORUM>          Quorum percentage (0-100) [default: 50]
      --base-out-path <BASE_OUT_PATH>  Quorum percentage (0-100) [default: ./base.yaml]
  -h, --help                     Print help information
  -V, --version                  Print version information
```

### Basic Usage

Run the tool with the YAML override files:

```bash
./yabe file1.yaml file2.yaml file3.yaml
```
This will compute the differences among the override files and generate:

* base.yaml: The common base configuration.
* file1_diff.yaml, file2_diff.yaml, file3_diff.yaml: The differences for each file.

### Inplace Modification

Use the -i or --inplace flag to modify the original override files with their differences:
```bash
./yabe -i -h helm_values.yaml file1.yaml file2.yaml file3.yaml
```

### Enable Debug Logging

Use the --debug flag to enable detailed debug logging:
```bash
./yabe --debug -h helm_values.yaml file1.yaml file2.yaml file3.yaml
```

## Examples

### Sample Input Files

_helm_values.yaml_
```yaml
settings:
  theme: dark
  notifications: true
  advanced:
    mode: auto
    level: 5
```

_file1.yaml_
```yaml
settings:
  theme: dark
  notifications: true
  advanced:
    mode: auto
    level: 5
```

_file2.yaml_
```yaml
settings:
  theme: light
  notifications: true
  advanced:
    mode: manual
    level: 5
```

_file3.yaml_
```yaml
settings:
  theme: dark
  notifications: false
  advanced:
    mode: auto
    level: 7
```

### Running the Tool
    
```bash
./yabe -h helm_values.yaml file1.yaml file2.yaml file3.yaml
```

### Expected Output
_base.yaml_
```yaml
settings:
  advanced:
    level: 5
```

_file1_diff.yaml_
(Empty file or not generated since there are no differences)

_file2_diff.yaml_
```yaml
settings:
  theme: light
  advanced:
    mode: manual
```

_file3_diff.yaml_
```yaml
settings:
  notifications: false
  advanced:
    level: 7
```

### Inplace Modification Example
Running with the -i flag:
```bash
./yabe -i -h helm_values.yaml file1.yaml file2.yaml file3.yaml
```

## Testing

The project includes a suite of tests to verify functionality. To run the tests:
```bash
cargo test
```
Ensure all tests pass to verify that the tool is functioning correctly.

### Project Structure
* _src/_
  * _lib.rs_: The library module containing core functionality.
  * _main.rs_: The main executable entry point.
  * _diff.rs_: Functions for computing diffs and common bases.
  * _deep_equal.rs_: Utility function for deep comparison of YAML values.
* _tests/_
  * _test_deep_equal.rs_: Tests for the deep_equal function.
  * _test_diff.rs_: Tests for compute_diff and diff_and_common_multiple functions.
  * _test_common.rs_: Common tests for the project.
* _Cargo.toml_: Project configuration file.