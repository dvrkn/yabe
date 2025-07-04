use std::error::Error;
use std::fs;
use std::path::Path;
use log::{info, warn};
use glob::glob;

/// Expand path patterns and filter by exclude patterns
pub fn expand_and_filter_files(
    input_files: Vec<String>,
    path_patterns: &[String],
    exclude_patterns: &[String],
) -> Result<Vec<String>, Box<dyn Error>> {
    // Process path patterns and add matching files to input_files
    let mut expanded_input_files = input_files;
    
    for pattern in path_patterns {
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
    if !exclude_patterns.is_empty() {
        let original_count = expanded_input_files.len();
        expanded_input_files.retain(|file_path| {
            for exclude_pattern in exclude_patterns {
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
    
    Ok(expanded_input_files)
}

/// Ensure output directory exists
pub fn ensure_output_dir(out_folder: &str) -> Result<(), Box<dyn Error>> {
    if !Path::new(out_folder).exists() {
        info!("Creating output directory: {}", out_folder);
        fs::create_dir_all(out_folder)?;
    }
    Ok(())
}