use clap::{Parser, Subcommand};
use serde::Deserialize;

/// Command-line arguments
#[derive(Parser)]
#[command(author, version, about = "YAML diff and merge tool for GitOps workflows", long_about = "A tool for diffing, merging, and organizing YAML files for GitOps workflows. Supports both individual file paths and glob patterns.")]
pub struct Args {
    /// Configuration file
    #[arg(long = "config", value_name = "CONFIG_FILE")]
    pub config: Option<String>,

    /// Enable debug logging
    #[arg(long = "debug")]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Legacy mode: Helm chart values file
    #[arg(short = 'r', long = "read-base", value_name = "READ_BASE")]
    pub read_only_base: Option<String>,

    /// Legacy mode: Base YAML file to merge with input files
    #[arg(short = 'b', long = "base", value_name = "WRITE_BASE")]
    pub base: Option<String>,

    /// Legacy mode: Input YAML files (optional if path patterns are provided)
    pub input_files: Vec<String>,

    /// Legacy mode: Path patterns to load YAML files (e.g., "*.yaml")
    #[arg(short = 'p', long = "path-pattern", value_name = "PATH_PATTERN")]
    pub path_patterns: Vec<String>,

    /// Legacy mode: Modify the original input files with diffs
    #[arg(short = 'i', long = "in-place")]
    pub inplace: bool,

    /// Legacy mode: Output folder
    #[arg(short = 'o', long = "out", default_value = "./out")]
    pub out_folder: String,

    /// Legacy mode: Quorum percentage (0-100)
    #[arg(short = 'q', long = "quorum", default_value_t = 51)]
    pub quorum: u8,

    /// Legacy mode: Base file output path
    #[arg(long = "base-out-path", default_value = "./base.yaml")]
    pub base_out_path: String,

    /// Legacy mode: Sort configuration file path
    #[arg(long = "sort-config-path", default_value = "./sort-config.yaml")]
    pub sort_config_path: String,

    /// Legacy mode: Sort only mode - only sort files without diffing
    #[arg(long = "sort-only")]
    pub sort_only: bool,

    /// Legacy mode: Exclude patterns to skip files (e.g., "*.terraform.yaml")
    #[arg(long = "exclude", value_name = "EXCLUDE_PATTERN")]
    pub exclude_patterns: Vec<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Sort YAML files based on configuration
    Sort {
        /// Input YAML files (optional if path patterns are provided)
        input_files: Vec<String>,

        /// Path patterns to load YAML files (e.g., "*.yaml")
        #[arg(short = 'p', long = "path-pattern", value_name = "PATH_PATTERN")]
        path_patterns: Vec<String>,

        /// Sort configuration file path
        #[arg(long = "sort-config", default_value = "./sort-config.yaml")]
        sort_config_path: String,

        /// Modify the original input files with sorted content
        #[arg(short = 'i', long = "in-place")]
        inplace: bool,

        /// Output folder
        #[arg(short = 'o', long = "out", default_value = "./out")]
        out_folder: String,

        /// Exclude patterns to skip files (e.g., "*.terraform.yaml")
        #[arg(long = "exclude", value_name = "EXCLUDE_PATTERN")]
        exclude_patterns: Vec<String>,
    },
    /// Separate common base from YAML files (diff/rebalance)
    Separate {
        /// Input YAML files (optional if path patterns are provided)
        input_files: Vec<String>,

        /// Path patterns to load YAML files (e.g., "*.yaml")
        #[arg(short = 'p', long = "path-pattern", value_name = "PATH_PATTERN")]
        path_patterns: Vec<String>,

        /// Helm chart values file
        #[arg(short = 'r', long = "read-base", value_name = "READ_BASE")]
        read_only_base: Option<String>,

        /// Base YAML file to merge with input files
        #[arg(short = 'b', long = "base", value_name = "WRITE_BASE")]
        base: Option<String>,

        /// Quorum percentage (0-100)
        #[arg(short = 'q', long = "quorum", default_value_t = 51)]
        quorum: u8,

        /// Base file output path
        #[arg(long = "base-out", default_value = "./base.yaml")]
        base_out_path: String,

        /// Sort configuration file path
        #[arg(long = "sort-config", default_value = "./sort-config.yaml")]
        sort_config_path: String,

        /// Modify the original input files with diffs
        #[arg(short = 'i', long = "in-place")]
        inplace: bool,

        /// Output folder
        #[arg(short = 'o', long = "out", default_value = "./out")]
        out_folder: String,

        /// Exclude patterns to skip files (e.g., "*.terraform.yaml")
        #[arg(long = "exclude", value_name = "EXCLUDE_PATTERN")]
        exclude_patterns: Vec<String>,
    },
}

#[derive(Deserialize)]
pub struct Config {
    pub read_only_base: Option<String>,
    pub base: Option<String>,
    pub input_files: Option<Vec<String>>,
    pub path_patterns: Option<Vec<String>>,
    pub path_pattern: Option<String>,
    pub inplace: Option<bool>,
    pub out_folder: Option<String>,
    pub debug: Option<bool>,
    pub quorum: Option<u8>,
    pub base_out_path: Option<String>,
    pub sort_config_path: Option<String>,
    pub sort_only: Option<bool>,
    pub exclude_patterns: Option<Vec<String>>,
}