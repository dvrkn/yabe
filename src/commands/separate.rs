use std::borrow::Cow;
use std::error::Error;
use std::fs;
use std::path::Path;
use log::info;
use yaml_rust2::{Yaml, YamlEmitter, YamlLoader};
use crate::diff::{compute_diff, diff_and_common_multiple};
use crate::merge::merge_yaml;
use crate::sorter::sort_yaml;
use crate::utils::{expand_and_filter_files, ensure_output_dir};

pub fn run_separate_command(
    input_files: Vec<String>,
    path_patterns: Vec<String>,
    read_only_base: Option<String>,
    base: Option<String>,
    quorum: u8,
    base_out_path: String,
    sort_config_path: String,
    inplace: bool,
    out_folder: String,
    exclude_patterns: Vec<String>,
) -> Result<(), Box<dyn Error>> {
    info!("Running separate command");
    
    let expanded_input_files = expand_and_filter_files(input_files, &path_patterns, &exclude_patterns)?;
    
    // Validate that either input_files or path_patterns are provided
    if expanded_input_files.is_empty() && path_patterns.is_empty() {
        eprintln!("Error: No input files or path patterns provided. Please specify either input files or path patterns.");
        std::process::exit(1);
    }
    
    // Validate that we have at least one file to process after expansion
    if expanded_input_files.is_empty() {
        eprintln!("Error: No files found matching the provided path patterns. Please check your patterns and try again.");
        std::process::exit(1);
    }

    let input_filenames = expanded_input_files;
    let quorum_percentage = (quorum as f64) / 100.0;

    // Ensure output directory exists
    ensure_output_dir(&out_folder)?;

    let config = if !sort_config_path.is_empty() {
        info!("Reading sort configuration file: {}", sort_config_path);
        let content = fs::read_to_string(&sort_config_path);
        if let Ok(content) = content {
            YamlLoader::load_from_str(&content)?.into_iter().next().unwrap_or(Yaml::Null)
        } else {
            log::warn!("Failed to read sort configuration file: {}", sort_config_path);
            Yaml::Null
        }
    } else {
        Yaml::Null
    };

    let read_only_base = if let Some(ref read_only_base) = read_only_base {
        info!("Reading helm values file: {}", read_only_base);
        let content = fs::read_to_string(read_only_base)?;
        YamlLoader::load_from_str(&content)?.into_iter().next()
    } else {
        None
    };

    // Read and parse the existing base file if provided
    let existing_base = if let Some(ref base_path) = base {
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
            log::warn!("No YAML documents in {}", filename);
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
        quorum
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
    if inplace {
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

    info!("Separate command completed successfully.");
    Ok(())
}