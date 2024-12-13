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

/// Command-line arguments
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
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

    /// Input YAML files
    input_files: Vec<String>,

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
}

#[derive(Deserialize)]
struct Config {
    read_only_base: Option<String>,
    base: Option<String>,
    input_files: Option<Vec<String>>,
    inplace: Option<bool>,
    out_folder: Option<String>,
    debug: Option<bool>,
    quorum: Option<u8>,
    base_out_path: Option<String>,
    sort_config_path: Option<String>,
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
    }

    // Initialize logger with appropriate level
    if args.debug {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    // Validate that input_files are provided
    if args.input_files.is_empty() {
        eprintln!("Error: No input files provided. Please specify input files via command-line arguments or in the configuration file.");
        std::process::exit(1);
    }

    info!("Starting the YAML diffing program.");

    let input_filenames = args.input_files;

    let quorum_percentage = (args.quorum as f64) / 100.0;

    let base_out_path = args.base_out_path;

    let out_folder = args.out_folder;

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