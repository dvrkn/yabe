
# YABE (YAml Base Extractor) - Multi-layer YAML organizer

The idea comes from the need to manage huge amount of YAML files in a GitOps environment. Especially when using ArgoCD multi-source apps with some common values and some overrides.
The tool helps to compute the common base configuration among multiple YAML files and generate differences for each file, reducing the duplication of configuration values. 
It also provides the ability to sort YAML content based on user-defined configuration.


## Features

- **Compute diffs:** Detect differences between YAML files.
- **Merge YAML files:** Combine YAML files with a base YAML, either from an existing file or dynamically computed.
- **Quorum-based diffing:** Extract common base YAML based on a quorum percentage.
- **Sort YAML content:** Sort keys in YAML files based on user-defined configuration.
- **Helm Values Integration:** Merge input YAML files with Helm values files.
- **In-place modification or output to new files.**

## Installation

```bash
cargo install yabe-gitops
```

## Usage

```bash
Usage: yabe [OPTIONS] <INPUT_FILES>...

Arguments:
  <INPUT_FILES>...  Input YAML files

Options:
  -r, --read-base <READ_BASE>                (Optional) Read-only base for values deduplication
  -b, --base <WRITE_BASE>                    (Optional) Common values of all input files, if not provided, will be computed
  -i, --in-place                             Modify the original input files with diffs
  -o, --out <OUT_FOLDER>                     Output folder for diff files [default: ./out]
      --debug                                Enable debug logging
  -q, --quorum <QUORUM>                      Quorum percentage (0-100) [default: 51]
      --base-out-path <BASE_OUT_PATH>        (Optional) Base file output path [default: ./base.yaml]
      --sort-config-path <SORT_CONFIG_PATH>  (Optional) Sort configuration file path [default: ./sort-config.yaml], if not provided, will not sort
  -h, --help                                 Print help
  -V, --version                              Print version
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
./yabe -i -r helm_values.yaml file1.yaml file2.yaml file3.yaml
```

### Enable Debug Logging

Use the --debug flag to enable detailed debug logging:
```bash
./yabe --debug -r helm_values.yaml file1.yaml file2.yaml file3.yaml
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

### Inplace Modification Example
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
