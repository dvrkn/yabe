use std::error::Error;
use clap::Parser;
use yabe::{Args, Commands, run_sort_command, run_separate_command};

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Initialize logger with appropriate level
    if args.debug {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    match args.command {
        Commands::Sort { 
            input_files, 
            path_patterns, 
            sort_config_path, 
            inplace, 
            out_folder, 
            exclude_patterns 
        } => {
            run_sort_command(input_files, path_patterns, sort_config_path, inplace, out_folder, exclude_patterns)
        }
        Commands::Separate { 
            config_file,
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
        } => {
            run_separate_command(config_file, input_files, path_patterns, read_only_base, base, quorum, base_out_path, sort_config_path, inplace, out_folder, exclude_patterns)
        }
    }
}