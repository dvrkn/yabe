use std::error::Error;
use std::fs;
use std::path::Path;
use log::{info, warn};
use yaml_rust2::{Yaml, YamlEmitter, YamlLoader};
use crate::sorter::sort_yaml;
use crate::utils::{expand_and_filter_files, ensure_output_dir};

pub fn run_sort_command(
    input_files: Vec<String>,
    path_patterns: Vec<String>,
    sort_config_path: String,
    inplace: bool,
    out_folder: String,
    exclude_patterns: Vec<String>,
) -> Result<(), Box<dyn Error>> {
    info!("Running sort command");
    
    let expanded_input_files = expand_and_filter_files(input_files, &path_patterns, &exclude_patterns)?;
    
    // Validate that we have files to process
    if expanded_input_files.is_empty() {
        eprintln!("Error: No input files found. Please specify either input files or path patterns.");
        std::process::exit(1);
    }

    // Read sort configuration
    let sort_config = if !sort_config_path.is_empty() && Path::new(&sort_config_path).exists() {
        info!("Reading sort configuration file: {}", sort_config_path);
        let content = fs::read_to_string(&sort_config_path)?;
        YamlLoader::load_from_str(&content)?.into_iter().next().unwrap_or(Yaml::Null)
    } else {
        eprintln!("Error: Sort configuration file is required but not found: {}", sort_config_path);
        std::process::exit(1);
    };

    if sort_config == Yaml::Null {
        eprintln!("Error: Sort configuration file is empty or invalid: {}", sort_config_path);
        std::process::exit(1);
    }

    // Ensure output directory exists if not in-place mode
    if !inplace {
        ensure_output_dir(&out_folder)?;
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
            if inplace {
                info!("Writing sorted content back to: {}", filename);
                fs::write(filename, out_str)?;
            } else {
                let input_path = Path::new(filename);
                let file_stem = input_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("sorted");
                let sorted_filename = format!("{}/{}_sorted.yaml", out_folder, file_stem);
                info!("Writing sorted content to: {}", sorted_filename);
                fs::write(&sorted_filename, out_str)?;
            }
        } else {
            warn!("No YAML documents found in {}", filename);
        }
    }

    info!("Sort command completed successfully.");
    Ok(())
}