# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Important Notes for Claude Code

* Always commit as current user, not as `claude`.

## Project Overview

YABE (YAml Base Extractor) is a GitOps YAML organizer written in Rust, published to crates.io as `yabe-gitops` (binary name: `yabe`, library name: `yabe`). It extracts the common base configuration among multiple YAML files and writes per-file diffs, reducing duplication in GitOps setups (e.g., ArgoCD multi-source apps with shared Helm values). It also sorts YAML content according to a user-defined config.

## Common Commands

```bash
cargo build                 # Build
cargo test                  # Run all tests
cargo test --test test_diff # Run one integration test file
cargo test test_name        # Run tests matching a name
cargo run -- --debug ...    # Run with debug logging

# Typical invocations
cargo run -- file1.yaml file2.yaml file3.yaml
cargo run -- --config config.yaml
cargo run -- -i -r helm_values.yaml file1.yaml file2.yaml
cargo run -- -p "**/*.yaml" --exclude "target" --exclude "*.tmp"
cargo run -- --sort-only --sort-config-path ./sort-config.yaml -p "**/*.yaml"
```

All tests are integration tests in `tests/` (there are no unit tests in `src/`). CI runs `cargo build --release` and `cargo test` on PRs.

## Architecture

### Modules
- **`main.rs`**: CLI parsing (clap), config-file loading, glob expansion, exclude filtering, and the two workflows (diff workflow and `sort_only_workflow`). All file I/O and YAML emitting lives here.
- **`lib.rs`**: Re-exports `diff`, `merge`, `sorter`, `deep_equal` modules.
- **`diff.rs`**: `compute_diff` (diff one file against a read-only base) and `diff_and_common_multiple` (recursive quorum-based common-base extraction across N files).
- **`merge.rs`**: `merge_yaml` — deep merge where override values win; only hashes merge recursively, everything else is replaced.
- **`sorter.rs`**: `sort_yaml` — recursively sorts hashes and arrays per the sort config.
- **`deep_equal.rs`**: Deep YAML equality used by the diff logic.

The core functions return `Cow<Yaml>` to avoid cloning unchanged subtrees — preserve this pattern when modifying them.

### Diff Workflow (main.rs)
1. Expand glob patterns, dedupe, apply exclude patterns.
2. If `--base` (write base) given, deep-merge it under each input file.
3. If `--read-base` given, `compute_diff` each merged doc against it (read-only base values are stripped, never written).
4. `diff_and_common_multiple` over the results extracts the common base by quorum and per-file diffs.
5. Sort output (if a sort config loads), then write `base.yaml` and either `<stem>_diff.yaml` files in the out folder, or overwrite the originals with `-i`.

### Key Behaviors (non-obvious)
- **Quorum**: a value goes into the base if ≥ `ceil(quorum% × file_count)` files agree on it. Files that disagree keep their value in their diff. Below quorum, the key stays in every file's diff.
- **Arrays are atomic** in `diff_and_common_multiple` (all-or-nothing into the base), but `compute_diff` diffs arrays element-wise when lengths match, emitting `Null` for unchanged elements.
- **In-place mode clears files**: with `-i`, a file with no remaining diff is truncated to empty, not deleted or left alone.
- **Config vs CLI precedence**: CLI wins, but this is implemented by "is the arg still at its default?" checks in `main()` — e.g. explicitly passing `--quorum 51` (the default) cannot override a config-file value. Any new option must be added to both the `Args` struct and the `Config` struct plus a merge block in `main()`.
- **Exclude patterns** match four ways (substring of the path, glob on the full path, glob on each path component, glob on the filename). This filtering logic is duplicated in `main()` and `sort_only_workflow()` — keep both in sync when changing it.

### Sort Config (`sort-config.yaml`)
- `sortKey`: key used to sort arrays of maps (e.g. sort by `name`).
- `preOrder`: hash keys listed here come first in that order; remaining keys are sorted alphabetically. Applied recursively with the same config at every level.

In the diff workflow the sort config is optional (missing file just skips sorting); in `--sort-only` mode it is required and the tool exits with an error if absent.

## Releasing
`publish.yaml` publishes to crates.io when a GitHub release is published. The `Cargo.toml` version must match the tag (`v<version>`), and the release notes are pulled from the matching `## [<version>]` section of `CHANGELOG.md` — bump both together.
