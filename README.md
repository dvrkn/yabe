# YABE (YAml Base Extractor) - Multi-layer YAML Organizer

YABE is a tool designed to help manage large amounts of YAML files in a GitOps environment, especially when using ArgoCD multi-source apps with common values and overrides. It computes the common base configuration among multiple YAML files and generates differences for each file, reducing duplication and simplifying configuration management. It also provides the ability to sort YAML content based on user-defined configurations.

## Features

- **Compute diffs:** Detect differences between YAML files using the `separate` command.
- **Merge YAML files:** Combine YAML files with a base YAML, either from an existing file or dynamically computed.
- **Quorum-based diffing:** Extract common base YAML based on a quorum percentage.
- **Sort YAML content:** Sort keys in YAML files based on user-defined configuration or alphabetically using the `sort` command.
- **Helm Values Integration:** Merge input YAML files with Helm values files.
- **In-place modification or output to new files.**
- **Flexible file selection:** Support for glob patterns and exclude patterns.
- **Command-based interface:** Clean separation between sorting and diffing operations.

## Installation

```bash
cargo install yabe-gitops
```

## Usage

YABE now uses subcommands to organize its functionality. Run `yabe --help` to see available commands:

```bash
Usage: yabe [OPTIONS] <COMMAND>

Commands:
  sort      Sort YAML files based on configuration
  separate  Separate common base from YAML files (diff/rebalance)
  help      Print this message or the help of the given subcommand(s)

Options:
      --debug   Enable debug logging
  -h, --help    Print help
  -V, --version Print version
```

### Sort Command

Sort YAML files based on configuration:

```bash
Usage: yabe sort [OPTIONS] [INPUT_FILES]...

Arguments:
  [INPUT_FILES]...  Input YAML files (optional if path patterns are provided)

Options:
  -p, --path-pattern <PATH_PATTERN>     Path patterns to load YAML files (e.g., "*.yaml")
      --sort-config <SORT_CONFIG_PATH>  Sort configuration file path [default: ./sort-config.yaml]
  -i, --in-place                        Modify the original input files with sorted content
  -o, --out <OUT_FOLDER>                Output folder [default: ./out]
      --exclude <EXCLUDE_PATTERN>       Exclude patterns to skip files (e.g., "*.terraform.yaml")
  -h, --help                            Print help
```

### Separate Command

Separate common base from YAML files (diff/rebalance):

```bash
Usage: yabe separate [OPTIONS] [INPUT_FILES]...

Arguments:
  [INPUT_FILES]...  Input YAML files (optional if path patterns are provided)

Options:
  -p, --path-pattern <PATH_PATTERN>     Path patterns to load YAML files (e.g., "*.yaml")
  -r, --read-base <READ_BASE>           Helm chart values file
  -b, --base <WRITE_BASE>               Base YAML file to merge with input files
  -q, --quorum <QUORUM>                 Quorum percentage (0-100) [default: 51]
      --base-out <BASE_OUT_PATH>        Base file output path [default: ./base.yaml]
      --sort-config <SORT_CONFIG_PATH>  Sort configuration file path [default: ./sort-config.yaml]
  -i, --in-place                        Modify the original input files with diffs
  -o, --out <OUT_FOLDER>                Output folder [default: ./out]
      --exclude <EXCLUDE_PATTERN>       Exclude patterns to skip files (e.g., "*.terraform.yaml")
  -h, --help                            Print help
```

**Note:** You must provide either input files or path patterns. If both are provided, all matching files will be processed.

### Basic Usage

#### Separating Common Base from YAML Files

Run the separate command with the YAML override files:

```bash
yabe separate file1.yaml file2.yaml file3.yaml
```

Or use path patterns to load multiple files:

```bash
yabe separate -p "*.yaml" -p "configs/*.yaml"
```

This will compute the differences among the override files and generate:

* base.yaml: The common base configuration.
* file1_diff.yaml, file2_diff.yaml, file3_diff.yaml: The differences for each file.

#### In-place Modification

Use the -i or --in-place flag to modify the original override files with their differences:
```bash
yabe separate -i -r helm_values.yaml file1.yaml file2.yaml file3.yaml
```

#### Enable Debug Logging

Use the --debug flag to enable detailed debug logging:
```bash
yabe --debug separate -r helm_values.yaml file1.yaml file2.yaml file3.yaml
```

#### Sort YAML Files

Use the sort command to sort YAML files based on configuration:

```bash
# Sort files and output to ./out directory
yabe sort --sort-config sort-config.yaml file1.yaml file2.yaml

# Sort files in-place (modify original files)
yabe sort --sort-config sort-config.yaml -i file1.yaml file2.yaml

# Sort files using path patterns
yabe sort --sort-config sort-config.yaml -p "*.yaml" -p "configs/*.yaml"

# Sort files to a specific output directory
yabe sort --sort-config sort-config.yaml -o ./sorted-files *.yaml

# Sort files recursively while excluding certain patterns
yabe sort --sort-config sort-config.yaml -p "**/*.yaml" --exclude "*.terraform.yaml" --exclude "*-template.yaml"

# Sort files in-place while excluding terraform files
yabe sort --sort-config sort-config.yaml -p "./envs/**/*.yaml" --exclude "*.terraform.yaml" -i

# Exclude files by directory name (any file with "target" in the path)
yabe sort --sort-config sort-config.yaml -p "**/*.yaml" --exclude "target"

# Exclude multiple patterns - files in build directories and temp files
yabe sort --sort-config sort-config.yaml -p "**/*.yaml" --exclude "target" --exclude "build" --exclude "*.tmp"

# Sort files with alphabetical ordering (when no sort config is available)
yabe sort file1.yaml file2.yaml  # Will apply default alphabetical sorting
```

### Exclude Patterns

The `--exclude` option supports flexible pattern matching to skip unwanted files during processing. You can specify multiple exclude patterns, and files matching any pattern will be skipped.

**Supported pattern types:**

1. **Substring matching**: Excludes files containing the pattern anywhere in the path
   ```bash
   --exclude "target"          # Excludes files with "target" in the path
   --exclude "node_modules"    # Excludes files with "node_modules" in the path
   ```

2. **Glob patterns**: Standard file matching patterns
   ```bash
   --exclude "*.tmp"           # Excludes all .tmp files
   --exclude ".*"              # Excludes hidden files
   --exclude "test_*.yaml"     # Excludes files starting with "test_"
   ```

3. **Path component matching**: Matches against individual directories or filenames
   ```bash
   --exclude "build"           # Excludes files in any "build" directory
   --exclude "dist"            # Excludes files in any "dist" directory
   ```

4. **Full path glob matching**: Complex path patterns
   ```bash
   --exclude "*/temp/*"        # Excludes files in any "temp" subdirectory
   --exclude "target/**"       # Excludes all files under target directory
   ```

**Examples:**
```bash
# Exclude multiple directory types (separate command)
yabe separate -p "**/*.yaml" --exclude "target" --exclude "node_modules" --exclude ".git"

# Exclude by file patterns and directories (separate command)
yabe separate -p "**/*.yaml" --exclude "*.terraform.yaml" --exclude "build" --exclude "dist"

# Complex exclusion for GitOps environments (sort command)
yabe sort -p "**/*.yaml" --exclude "target" --exclude ".argocd" --exclude "*.secret.yaml"
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
yabe separate -r helm_values.yaml file1.yaml file2.yaml file3.yaml
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
yabe separate -i -r helm_values.yaml file1.yaml file2.yaml file3.yaml
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
