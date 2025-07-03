use std::borrow::Cow;
use std::error::Error;
use std::fs;
use std::path::Path;

use clap::Parser;
use log::{info, warn};
use yaml_rust2::{Yaml, YamlEmitter, YamlLoader};
use yabe::diff::{compute_diff, diff_and_common_multiple};
use yabe::merge::merge_yaml;
use yabe::sorter::sort_yaml;
use serde::Deserialize;
use serde_yaml;
use glob::glob;

/// Command-line arguments
#[derive(Parser)]
#[command(author, version, about = "YAML diff and merge tool for GitOps workflows", long_about = "A tool for diffing, merging, and organizing YAML files for GitOps workflows. Supports both individual file paths and glob patterns.")]
struct Args {
    /// Configuration file
    #[arg(long = "config", value_name = "CONFIG_FILE")]
    config: Option<String>,

    /// Helm chart values file
    #[arg(short = 'r', long = "read-base", value_name = "READ_BASE")]
    read_only_base: Option<String>,

    /// Base YAML file to merge with input files
    #[arg(short = 'b', long = "base", value_name = "WRITE_BASE")]
    base: Option<String>,

    /// Input YAML files (optional if path patterns are provided)
    input_files: Vec<String>,

    /// Path patterns to load YAML files (e.g., "*.yaml")
    #[arg(short = 'p', long = "path-pattern", value_name = "PATH_PATTERN")]
    path_patterns: Vec<String>,

    /// Modify the original input files with diffs
    #[arg(short = 'i', long = "in-place")]
    inplace: bool,

    /// Output folder
    #[arg(short = 'o', long = "out", default_value = "./out")]
    out_folder: String,

    /// Enable debug logging
    #[arg(long = "debug")]
    debug: bool,

    /// Quorum percentage (0-100)
    #[arg(short = 'q', long = "quorum", default_value_t = 51)]
    quorum: u8,

    /// Base file output path
    #[arg(long = "base-out-path", default_value = "./base.yaml")]
    base_out_path: String,

    /// Sort configuration file path
    #[arg(long = "sort-config-path", default_value = "./sort-config.yaml")]
    sort_config_path: String,

    /// Sort only mode - only sort files without diffing
    #[arg(long = "sort-only")]
    sort_only: bool,

    /// Exclude patterns to skip files (e.g., "*.terraform.yaml")
    #[arg(long = "exclude", value_name = "EXCLUDE_PATTERN")]
    exclude_patterns: Vec<String>,
}

#[derive(Deserialize)]
struct Config {
    read_only_base: Option<String>,
    base: Option<String>,
    input_files: Option<Vec<String>>,
    path_patterns: Option<Vec<String>>,
    path_pattern: Option<String>,
    inplace: Option<bool>,
    out_folder: Option<String>,
    debug: Option<bool>,
    quorum: Option<u8>,
    base_out_path: Option<String>,
    sort_config_path: Option<String>,
    sort_only: Option<bool>,
    exclude_patterns: Option<Vec<String>>,
}

fn sort_only_workflow(args: &Args) -> Result<(), Box<dyn Error>> {
    info!("Running in sort-only mode");

    // Process path patterns and add matching files to input_files
    let mut expanded_input_files = args.input_files.clone();
    
    for pattern in &args.path_patterns {
        info!("Expanding path pattern: {}", pattern);
        match glob(pattern) {
            Ok(paths) => {
                for entry in paths {
                    match entry {
                        Ok(path) => {
                            if path.is_file() {
                                if let Some(path_str) = path.to_str() {
                                    info!("Found matching file: {}", path_str);
                                    expanded_input_files.push(path_str.to_string());
                                }
                            }
                        }
                        Err(e) => warn!("Error matching path: {}", e),
                    }
                }
            }
            Err(e) => warn!("Invalid glob pattern '{}': {}", pattern, e),
        }
    }
    
    // Remove duplicates from expanded_input_files
    expanded_input_files.sort();
    expanded_input_files.dedup();
    
    // Filter out excluded files
    if !args.exclude_patterns.is_empty() {
        let original_count = expanded_input_files.len();
        expanded_input_files.retain(|file_path| {
            for exclude_pattern in &args.exclude_patterns {
                // Check if the file path contains the exclude pattern as a substring
                if file_path.contains(exclude_pattern) {
                    info!("Excluding file: {} (path contains pattern: {})", file_path, exclude_pattern);
                    return false;
                }
                
                // Check if the exclude pattern matches as a glob pattern against the full path
                if let Ok(pattern) = glob::Pattern::new(exclude_pattern) {
                    if pattern.matches(file_path) {
                        info!("Excluding file: {} (path matches glob pattern: {})", file_path, exclude_pattern);
                        return false;
                    }
                }
                
                // Check if any component of the path matches the pattern
                for component in Path::new(file_path).components() {
                    if let Some(component_str) = component.as_os_str().to_str() {
                        if let Ok(pattern) = glob::Pattern::new(exclude_pattern) {
                            if pattern.matches(component_str) {
                                info!("Excluding file: {} (path component '{}' matches pattern: {})", file_path, component_str, exclude_pattern);
                                return false;
                            }
                        }
                    }
                }
                
                // Check if the filename matches the pattern directly
                if let Some(filename) = Path::new(file_path).file_name() {
                    if let Some(filename_str) = filename.to_str() {
                        if let Ok(pattern) = glob::Pattern::new(exclude_pattern) {
                            if pattern.matches(filename_str) {
                                info!("Excluding file: {} (filename matches pattern: {})", file_path, exclude_pattern);
                                return false;
                            }
                        }
                    }
                }
            }
            true
        });
        let excluded_count = original_count - expanded_input_files.len();
        if excluded_count > 0 {
            info!("Excluded {} files based on exclude patterns", excluded_count);
        }
    }
    
    // Validate that we have files to process
    if expanded_input_files.is_empty() {
        eprintln!("Error: No input files found for sort-only mode. Please specify either input files or path patterns.");
        std::process::exit(1);
    }

    // Read sort configuration
    let sort_config = if !args.sort_config_path.is_empty() && Path::new(&args.sort_config_path).exists() {
        info!("Reading sort configuration file: {}", args.sort_config_path);
        let content = fs::read_to_string(&args.sort_config_path)?;
        YamlLoader::load_from_str(&content)?.into_iter().next().unwrap_or(Yaml::Null)
    } else {
        eprintln!("Error: Sort configuration file is required for sort-only mode but not found: {}", args.sort_config_path);
        std::process::exit(1);
    };

    if sort_config == Yaml::Null {
        eprintln!("Error: Sort configuration file is empty or invalid: {}", args.sort_config_path);
        std::process::exit(1);
    }

    // Ensure output directory exists if not in-place mode
    if !args.inplace {
        if !Path::new(&args.out_folder).exists() {
            info!("Creating output directory: {}", args.out_folder);
            fs::create_dir_all(&args.out_folder)?;
        }
    }

    // Process each input file
    for filename in &expanded_input_files {
        info!("Sorting file: {}", filename);
        
        // Read and parse the YAML file
        let content = fs::read_to_string(filename)?;
        if let Some(doc) = YamlLoader::load_from_str(&content)?.into_iter().next() {
            // Sort the YAML document
            let sorted_yaml = sort_yaml(&doc, &sort_config);
            
            // Convert to string
            let mut out_str = String::new();
            {
                let mut emitter = YamlEmitter::new(&mut out_str);
                emitter.dump(&sorted_yaml)?;
            }
            out_str = out_str.trim_start_matches("---\n").to_string();
            out_str.push('\n');
            
            // Write the sorted content
            if args.inplace {
                info!("Writing sorted content back to: {}", filename);
                fs::write(filename, out_str)?;
            } else {
                let input_path = Path::new(filename);
                let file_stem = input_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("sorted");
                let sorted_filename = format!("{}/{}_sorted.yaml", args.out_folder, file_stem);
                info!("Writing sorted content to: {}", sorted_filename);
                fs::write(&sorted_filename, out_str)?;
            }
        } else {
            warn!("No YAML documents found in {}", filename);
        }
    }

    info!("Sort-only workflow completed successfully.");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = Args::parse();

    if let Some(config_path) = args.config.as_ref() {
        // Read the configuration file
        let config_content = fs::read_to_string(config_path)?;
        let config: Config = serde_yaml::from_str(&config_content)?;

        // Override args with config values if they are not provided via command-line
        if args.read_only_base.is_none() {
            args.read_only_base = config.read_only_base;
        }

        if args.base.is_none() {
            args.base = config.base;
        }

        if args.input_files.is_empty() {
            if let Some(input_files) = config.input_files {
                args.input_files = input_files;
            }
        }

        if args.path_patterns.is_empty() {
            if let Some(path_patterns) = config.path_patterns {
                args.path_patterns = path_patterns;
            } else if let Some(path_pattern) = config.path_pattern.as_ref() {
                args.path_patterns.push(path_pattern.clone());
            }
        }

        if !args.inplace {
            if let Some(inplace) = config.inplace {
                args.inplace = inplace;
            }
        }

        if args.out_folder == "./out" {
            if let Some(out_folder) = config.out_folder {
                args.out_folder = out_folder;
            }
        }

        if !args.debug {
            if let Some(debug) = config.debug {
                args.debug = debug;
            }
        }

        if args.quorum == 51 {
            if let Some(quorum) = config.quorum {
                args.quorum = quorum;
            }
        }

        if args.base_out_path == "./base.yaml" {
            if let Some(base_out_path) = config.base_out_path {
                args.base_out_path = base_out_path;
            }
        }

        if args.sort_config_path == "./sort-config.yaml" {
            if let Some(sort_config_path) = config.sort_config_path {
                args.sort_config_path = sort_config_path;
            }
        }

        if !args.sort_only {
            if let Some(sort_only) = config.sort_only {
                args.sort_only = sort_only;
            }
        }

        if args.exclude_patterns.is_empty() {
            if let Some(exclude_patterns) = config.exclude_patterns {
                args.exclude_patterns = exclude_patterns;
            }
        }
    }

    // Initialize logger with appropriate level
    if args.debug {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    // Handle sort-only mode
    if args.sort_only {
        return sort_only_workflow(&args);
    }

    // Process path patterns and add matching files to input_files
    let mut expanded_input_files = args.input_files.clone();
    
    for pattern in &args.path_patterns {
        info!("Expanding path pattern: {}", pattern);
        match glob(pattern) {
            Ok(paths) => {
                for entry in paths {
                    match entry {
                        Ok(path) => {
                            if path.is_file() {
                                if let Some(path_str) = path.to_str() {
                                    info!("Found matching file: {}", path_str);
                                    expanded_input_files.push(path_str.to_string());
                                }
                            }
                        }
                        Err(e) => warn!("Error matching path: {}", e),
                    }
                }
            }
            Err(e) => warn!("Invalid glob pattern '{}': {}", pattern, e),
        }
    }
    
    // Remove duplicates from expanded_input_files
    expanded_input_files.sort();
    expanded_input_files.dedup();
    
    // Filter out excluded files
    if !args.exclude_patterns.is_empty() {
        let original_count = expanded_input_files.len();
        expanded_input_files.retain(|file_path| {
            for exclude_pattern in &args.exclude_patterns {
                // Check if the file path contains the exclude pattern as a substring
                if file_path.contains(exclude_pattern) {
                    info!("Excluding file: {} (path contains pattern: {})", file_path, exclude_pattern);
                    return false;
                }
                
                // Check if the exclude pattern matches as a glob pattern against the full path
                if let Ok(pattern) = glob::Pattern::new(exclude_pattern) {
                    if pattern.matches(file_path) {
                        info!("Excluding file: {} (path matches glob pattern: {})", file_path, exclude_pattern);
                        return false;
                    }
                }
                
                // Check if any component of the path matches the pattern
                for component in Path::new(file_path).components() {
                    if let Some(component_str) = component.as_os_str().to_str() {
                        if let Ok(pattern) = glob::Pattern::new(exclude_pattern) {
                            if pattern.matches(component_str) {
                                info!("Excluding file: {} (path component '{}' matches pattern: {})", file_path, component_str, exclude_pattern);
                                return false;
                            }
                        }
                    }
                }
                
                // Check if the filename matches the pattern directly
                if let Some(filename) = Path::new(file_path).file_name() {
                    if let Some(filename_str) = filename.to_str() {
                        if let Ok(pattern) = glob::Pattern::new(exclude_pattern) {
                            if pattern.matches(filename_str) {
                                info!("Excluding file: {} (filename matches pattern: {})", file_path, exclude_pattern);
                                return false;
                            }
                        }
                    }
                }
            }
            true
        });
        let excluded_count = original_count - expanded_input_files.len();
        if excluded_count > 0 {
            info!("Excluded {} files based on exclude patterns", excluded_count);
        }
    }
    
    // Validate that either input_files or path_patterns are provided
    if expanded_input_files.is_empty() && args.path_patterns.is_empty() {
        eprintln!("Error: No input files or path patterns provided. Please specify either input files or path patterns via command-line arguments or in the configuration file.");
        std::process::exit(1);
    }
    
    // Validate that we have at least one file to process after expansion
    if expanded_input_files.is_empty() {
        eprintln!("Error: No files found matching the provided path patterns. Please check your patterns and try again.");
        std::process::exit(1);
    }

    info!("Starting the YAML diffing program.");

    let input_filenames = expanded_input_files;

    let quorum_percentage = (args.quorum as f64) / 100.0;

    let base_out_path = args.base_out_path;

    let out_folder = args.out_folder;

    // Ensure output directory exists
    if !Path::new(&out_folder).exists() {
        info!("Creating output directory: {}", out_folder);
        fs::create_dir_all(&out_folder)?;
    }

    let config = if !args.sort_config_path.is_empty() {
        info!("Reading sort configuration file: {}", args.sort_config_path);
        let content = fs::read_to_string(&args.sort_config_path);
        if let Ok(content) = content {
            YamlLoader::load_from_str(&content)?.into_iter().next().unwrap_or(Yaml::Null)
        } else {
            warn!("Failed to read sort configuration file: {}", args.sort_config_path);
            Yaml::Null
        }
    } else {
        Yaml::Null
    };

    let read_only_base = if let Some(ref read_only_base) = args.read_only_base {
        info!("Reading helm values file: {}", read_only_base);
        let content = fs::read_to_string(read_only_base)?;
        YamlLoader::load_from_str(&content)?.into_iter().next()
    } else {
        None
    };

    // Read and parse the existing base file if provided
    let existing_base = if let Some(ref base_path) = args.base {
        info!("Reading existing base YAML file: {}", base_path);
        let content = fs::read_to_string(base_path)?;
        YamlLoader::load_from_str(&content)?.into_iter().next()
    } else {
        None
    };

    // Read and parse each YAML input file into an object
    let mut all_docs = Vec::new();
    for filename in &input_filenames {
        info!("Reading input file: {}", filename);
        let content = fs::read_to_string(filename)?;
        if let Some(doc) = YamlLoader::load_from_str(&content)?.into_iter().next() {
            all_docs.push(doc);
        } else {
            warn!("No YAML documents in {}", filename);
        }
    }

    // Merge existing base with each input file if existing base is provided
    let merged_objs: Vec<Cow<Yaml>> = if let Some(ref base) = existing_base {
        input_filenames
            .iter()
            .zip(all_docs.iter())
            .map(|(filename, obj)| {
                let merged = merge_yaml(base, obj);
                info!("Merged base with input file: {}", filename);
                merged
            })
            .collect()
    } else {
        // No existing base; use objs as merged_objs
        all_docs.iter().map(|doc| Cow::Borrowed(doc)).collect()
    };

    // Compute diffs between each merged object and read-only base
    let diffs: Vec<_> = if let Some(ref helm) = read_only_base {
        info!("Computing diffs between merged files and helm values.");
        merged_objs
            .iter()
            .map(|obj| compute_diff(obj.as_ref(), helm).unwrap_or_else(|| Cow::Owned(Yaml::Null)))
            .collect()
    } else {
        // No read-only base provided values; use merged_objs as diffs
        merged_objs.clone()
    };

    // Now compute common base and per-file diffs among the diffs
    let diffs_refs: Vec<&Yaml> = diffs.iter().map(|cow| cow.as_ref()).collect();
    info!(
        "Computing common base and per-file diffs among the diffs with quorum {}%.",
        args.quorum
    );
    let (base, per_file_diffs) = diff_and_common_multiple(&diffs_refs, quorum_percentage);

    // Process the base YAML if it exists
    if let Some(base_yaml) = base {
        let processed_yaml = if config != Yaml::Null {
            sort_yaml(base_yaml.as_ref(), &config)
        } else {
            Cow::Borrowed(base_yaml.as_ref())
        };

        info!("Writing base YAML to {}", base_out_path);
        let mut out_str = String::new();
        {
            let mut emitter = YamlEmitter::new(&mut out_str);
            emitter.dump(&processed_yaml)?;
        }
        out_str = out_str.trim_start_matches("---\n").to_string();
        out_str.push('\n');
        fs::write(base_out_path.as_str(), out_str)?;
        info!("Base YAML written to {}", base_out_path);
    } else {
        info!("No base YAML to write.");
    }

    // Determine whether to write diffs to original files or new files
    if args.inplace {
        info!("Inplace mode enabled. Modifying original files.");
        for (i, diff) in per_file_diffs.iter().enumerate() {
            if let Some(diff_yaml) = diff {
                let processed_diff = if config != Yaml::Null {
                    sort_yaml(diff_yaml.as_ref(), &config)
                } else {
                    Cow::Borrowed(diff_yaml.as_ref())
                };

                info!("Writing diff back to original file: {}", input_filenames[i]);
                let mut out_str = String::new();
                {
                    let mut emitter = YamlEmitter::new(&mut out_str);
                    emitter.dump(&processed_diff)?;
                }
                out_str = out_str.trim_start_matches("---\n").to_string();
                out_str.push('\n');
                fs::write(&input_filenames[i], out_str)?;
                info!(
                    "Difference written back to original file {}",
                    input_filenames[i]
                );
            } else {
                // If there is no diff, remove the content of the file
                info!("No diff for {}; clearing file content.", input_filenames[i]);
                fs::write(&input_filenames[i], "")?;
                info!(
                    "No difference for {}; file content cleared.",
                    input_filenames[i]
                );
            }
        }
    } else {
        info!("Writing diffs to new files.");
        for (i, diff) in per_file_diffs.iter().enumerate() {
            if let Some(diff_yaml) = diff {
                let processed_diff = if config != Yaml::Null {
                    sort_yaml(diff_yaml.as_ref(), &config)
                } else {
                    Cow::Borrowed(diff_yaml.as_ref())
                };

                info!("Writing diff for {} to new file.", input_filenames[i]);
                let mut out_str = String::new();
                {
                    let mut emitter = YamlEmitter::new(&mut out_str);
                    emitter.dump(&processed_diff)?;
                }

                let input_path = Path::new(&input_filenames[i]);
                let file_stem = input_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("diff");
                let diff_filename = format!("{}/{}_diff.yaml", out_folder, file_stem);
                out_str = out_str.trim_start_matches("---\n").to_string();
                out_str.push('\n');
                fs::write(&diff_filename, out_str)?;
                info!(
                    "Difference for {} written to {}",
                    input_filenames[i], diff_filename
                );
            } else {
                info!("No diff for {}; not writing a diff file.", input_filenames[i]);
            }
        }
    }

    info!("Program completed successfully.");
    Ok(())
}