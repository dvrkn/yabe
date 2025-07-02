# YABE (YAml Base Extractor) - Multi-layer YAML Organizer

YABE is a tool designed to help manage large amounts of YAML files in a GitOps environment, especially when using ArgoCD multi-source apps with common values and overrides. It computes the common base configuration among multiple YAML files and generates differences for each file, reducing duplication and simplifying configuration management. It also provides the ability to sort YAML content based on user-defined configurations.

## Features

- **Compute diffs:** Detect differences between YAML files.
- **Merge YAML files:** Combine YAML files with a base YAML, either from an existing file or dynamically computed.
- **Quorum-based diffing:** Extract common base YAML based on a quorum percentage.
- **Sort YAML content:** Sort keys in YAML files based on user-defined configuration.
- **Sort-only mode:** Sort YAML files without performing any diffing operations.
- **Helm Values Integration:** Merge input YAML files with Helm values files.
- **In-place modification or output to new files.**
- **Configuration File Support:** Run the tool using a configuration file to simplify usage in automated workflows.

## Installation

```bash
cargo install yabe-gitops
```

## Usage

```bash
Usage: yabe [OPTIONS] [INPUT_FILES]...

Arguments:
  [INPUT_FILES]...  Input YAML files (optional if path patterns are provided)

Options:
  -r, --read-base <READ_BASE>                (Optional) Read-only base for values deduplication
  -b, --base <WRITE_BASE>                    (Optional) Common values of all input files, if not provided, will be computed
  -p, --path-pattern <PATH_PATTERN>          Path patterns to load YAML files (e.g., "*.yaml")
  -i, --in-place                             Modify the original input files with diffs
  -o, --out <OUT_FOLDER>                     Output folder for diff files [default: ./out]
      --debug                                Enable debug logging
  -q, --quorum <QUORUM>                      Quorum percentage (0-100) [default: 51]
      --base-out-path <BASE_OUT_PATH>        (Optional) Base file output path [default: ./base.yaml]
      --sort-config-path <SORT_CONFIG_PATH>  (Optional) Sort configuration file path [default: ./sort-config.yaml], if not provided, will not sort
      --sort-only                            Sort only mode - only sort files without diffing
      --config <CONFIG_FILE>                 (Optional) Configuration file
  -h, --help                                 Print help
  -V, --version                              Print version
```

**Note:** You must provide either input files or path patterns. If both are provided, all matching files will be processed.

### Basic Usage

Run the tool with the YAML override files:

```bash
./yabe file1.yaml file2.yaml file3.yaml
```

Or use path patterns to load multiple files:

```bash
./yabe -p "*.yaml" -p "configs/*.yaml"
```

This will compute the differences among the override files and generate:

* base.yaml: The common base configuration.
* file1_diff.yaml, file2_diff.yaml, file3_diff.yaml: The differences for each file.

### In-place Modification

Use the -i or --in-place flag to modify the original override files with their differences:
```bash
./yabe -i -r helm_values.yaml file1.yaml file2.yaml file3.yaml
```

### Enable Debug Logging

Use the --debug flag to enable detailed debug logging:
```bash
./yabe --debug -r helm_values.yaml file1.yaml file2.yaml file3.yaml
```

### Sort Only Mode

Use the --sort-only flag to only sort YAML files without performing any diffing operations:

```bash
# Sort files and output to ./out directory
./yabe --sort-only --sort-config-path sort-config.yaml file1.yaml file2.yaml

# Sort files in-place (modify original files)
./yabe --sort-only --sort-config-path sort-config.yaml -i file1.yaml file2.yaml

# Sort files using path patterns
./yabe --sort-only --sort-config-path sort-config.yaml -p "*.yaml" -p "configs/*.yaml"

# Sort files to a specific output directory
./yabe --sort-only --sort-config-path sort-config.yaml -o ./sorted-files *.yaml
```

### Using Configuration File

You can also use a configuration file to specify options:

```yaml
# config.yaml
read_only_base: "path/to/read_only_base.yaml"
base: "path/to/base.yaml"
# Either input_files or path_patterns/path_pattern (or both) must be specified
input_files:
  - "input1.yaml"
  - "input2.yaml"
# You can use either path_patterns (array) or path_pattern (single string)
path_patterns:
  - "*.yaml"
  - "configs/*.yaml"
# OR
# path_pattern: "*.yaml"
inplace: true
out_folder: "./output"
debug: false
quorum: 60
base_out_path: "./base_output.yaml"
sort_config_path: "./sort_config.yaml"
sort_only: false  # Set to true for sort-only mode
```

Then run the tool with:

```bash
./yabe --config config.yaml
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
./yabe -r helm_values.yaml file1.yaml file2.yaml file3.yaml
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

### In-place Modification Example
Running with the -i flag:
```bash
./yabe -i -r helm_values.yaml file1.yaml file2.yaml file3.yaml
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
  * _sorter.rs_: Functions for sorting YAML content.
* _tests/_
  * _test_deep_equal.rs_: Tests for the deep_equal function.
  * _test_diff.rs_: Tests for compute_diff and diff_and_common_multiple functions.
  * _test_common.rs_: Common tests for the project.
  * _test_sorter.rs_: Tests for the sorter functions.
* _Cargo.toml_: Project configuration file.
* _sort-config.yaml_: Configuration file for sorting YAML content.
