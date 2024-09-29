pub mod deep_equal;
pub mod diff;
pub mod merge;
pub mod sorter;

// Re-export functions if needed
pub use diff::{compute_diff, diff_and_common_multiple};