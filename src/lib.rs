//! pb - A CLI progress bar tool for time-based visualization
//!
//! This library provides the core functionality for the pb CLI tool,
//! including time parsing, progress calculation, and error handling.

pub mod cli;
pub mod error;
pub mod interaction;
pub mod progress;
pub mod progress_bar;

// Re-export commonly used types
pub use anyhow::{Context, Result as AnyhowResult};
pub use cli::{build_command, Args};
pub use error::{PbError, PbResult};
pub use progress_bar::{
    calculate_progress, format_duration, render_colored_progress_bar,
    render_colored_progress_bar_with_time, render_progress_bar, render_progress_bar_with_time,
};
