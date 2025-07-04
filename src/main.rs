use std::error::Error;
use clap::Parser;
use yabe::{Args, Commands, run_sort_command, run_separate_command, run_legacy_main};

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Initialize logger with appropriate level
    if args.debug {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    match args.command {
        Some(Commands::Sort { 
            input_files, 
            path_patterns, 
            sort_config_path, 
            inplace, 
            out_folder, 
            exclude_patterns 
        }) => {
            run_sort_command(input_files, path_patterns, sort_config_path, inplace, out_folder, exclude_patterns)
        }
        Some(Commands::Separate { 
            input_files, 
            path_patterns, 
            read_only_base, 
            base, 
            quorum, 
            base_out_path, 
            sort_config_path, 
            inplace, 
            out_folder, 
            exclude_patterns 
        }) => {
            run_separate_command(input_files, path_patterns, read_only_base, base, quorum, base_out_path, sort_config_path, inplace, out_folder, exclude_patterns)
        }
        None => {
            // Legacy mode - use the existing logic
            run_legacy_main(args)
        }
    }
}