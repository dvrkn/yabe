pub mod deep_equal;
pub mod diff;
pub mod merge;
pub mod sorter;
pub mod cli;
pub mod commands;
pub mod utils;

pub use diff::{compute_diff, diff_and_common_multiple};
pub use cli::{Args, Commands};
pub use commands::sort::run_sort_command;
pub use commands::separate::run_separate_command;