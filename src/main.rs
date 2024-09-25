use std::borrow::Cow;
use std::error::Error;
use std::fs;
use std::path::Path;

use clap::Parser;
use log::{info, warn};
use yaml_rust2::{Yaml, YamlEmitter, YamlLoader};

use yabe::diff::{compute_diff, diff_and_common_multiple};

/// Command-line arguments
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Helm chart values file
    #[arg(short = 'h', long = "helm", value_name = "HELM_VALUES_FILE")]
    helm_values: Option<String>,

    /// Input YAML files
    #[arg(required = true)]
    input_files: Vec<String>,

    /// Modify the original input files with diffs
    #[arg(short = 'i', long = "inplace")]
    inplace: bool,

    /// Enable debug logging
    #[arg(long = "debug")]
    debug: bool,

    /// Quorum percentage (0-100)
    #[arg(short = 'q', long = "quorum", default_value_t = 50)]
    quorum: u8,

    /// Base file output path
    #[arg(long = "base-out-path", default_value = "./base.yaml")]
    base_out_path: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command-line arguments
    let args = Args::parse();

    // Initialize the logger
    if args.debug {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    // Log the start of the program
    info!("Starting the YAML diffing program.");

    // List of input YAML filenames from command-line arguments
    let input_filenames = args.input_files;

    // Compute the quorum percentage from the command-line argument
    let quorum_percentage = (args.quorum as f64) / 100.0;

    // Get base output path
    let base_out_path = args.base_out_path;

    // Read and parse the helm chart values file if provided
    let helm_values = if let Some(ref helm_filename) = args.helm_values {
        info!("Reading helm values file: {}", helm_filename);
        let content = fs::read_to_string(helm_filename)?;
        let docs = YamlLoader::load_from_str(&content)?;
        if docs.is_empty() {
            warn!("No YAML documents in helm values file {}", helm_filename);
            None
        } else {
            Some(docs[0].clone())
        }
    } else {
        None
    };

    // Read and parse each YAML file into an object
    let mut all_docs = Vec::new();
    let mut objs = Vec::new();
    for filename in &input_filenames {
        info!("Reading input file: {}", filename);
        let content = fs::read_to_string(filename)?;
        let docs = YamlLoader::load_from_str(&content)?;
        if docs.is_empty() {
            warn!("No YAML documents in {}", filename);
            continue; // Skip empty files
        }
        all_docs.push(docs);
    }

    // Now collect references to the first document of each file
    for docs in &all_docs {
        objs.push(&docs[0]);
    }

    // Compute diffs between each obj and helm values
    let diffs: Vec<_> = if let Some(helm) = helm_values.as_ref() {
        info!("Computing diffs between override files and helm values.");
        objs.iter()
            .map(|obj| compute_diff(obj, helm).unwrap_or_else(|| Cow::Owned(Yaml::Null)))
            .collect()
    } else {
        // No helm values; use objs as diffs
        objs.iter().map(|obj| Cow::Borrowed(*obj)).collect()
    };

    // Now compute common base and per-file diffs among the diffs
    let diffs_refs: Vec<&Yaml> = diffs.iter().map(|cow| cow.as_ref()).collect();
    info!(
        "Computing common base and per-file diffs among the diffs with quorum {}%.",
        args.quorum
    );
    let (base, per_file_diffs) = diff_and_common_multiple(&diffs_refs, quorum_percentage);

    // Write the base YAML file if it exists
    if let Some(base_yaml) = base {
        info!("Writing base YAML to {}", base_out_path);
        let mut out_str = String::new();
        {
            let mut emitter = YamlEmitter::new(&mut out_str);
            emitter.dump(&base_yaml)?;
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
        // Modify the original input files with the diffs
        for (i, diff) in per_file_diffs.iter().enumerate() {
            if let Some(diff_yaml) = diff {
                info!("Writing diff back to original file: {}", input_filenames[i]);
                let mut out_str = String::new();
                {
                    let mut emitter = YamlEmitter::new(&mut out_str);
                    emitter.dump(diff_yaml)?;
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
        // Write diff files with modified names
        for (i, diff) in per_file_diffs.iter().enumerate() {
            if let Some(diff_yaml) = diff {
                info!("Writing diff for {} to new file.", input_filenames[i]);
                let mut out_str = String::new();
                {
                    let mut emitter = YamlEmitter::new(&mut out_str);
                    emitter.dump(diff_yaml)?;
                }
                // Extract the base name of the input file
                let input_path = Path::new(&input_filenames[i]);
                let file_stem = input_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("diff");
                // Create the diff filename by appending '_diff.yaml'
                let diff_filename = format!("{}_diff.yaml", file_stem);
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