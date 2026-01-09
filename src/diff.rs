pub mod detector;
pub mod types;

pub use detector::detect_diff;
pub use types::{Change, DiffResult};
