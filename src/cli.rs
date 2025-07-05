use clap::{Parser, Subcommand};

/// Command-line arguments
#[derive(Parser)]
#[command(author, version, about = "YAML diff and merge tool for GitOps workflows", long_about = "A tool for diffing, merging, and organizing YAML files for GitOps workflows. Supports both individual file paths and glob patterns.")]
pub struct Args {
    /// Enable debug logging
    #[arg(long = "debug")]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Commands,
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
        /// Configuration file path
        #[arg(short = 'c', long = "config", value_name = "CONFIG_FILE")]
        config_file: Option<String>,

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

