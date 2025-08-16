//! pb - A CLI progress bar tool for time-based visualization
//!
//! This library provides the core functionality for the pb CLI tool,
//! including time parsing, progress calculation, and error handling.

pub mod cli;
pub mod progress_bar;
pub mod theme;

// Re-export commonly used types
pub use cli::{build_command, Args};
pub use progress_bar::ProgressBar;
pub use theme::{DefaultTheme, RenderContext, RetroTheme, Theme, ThemeRegistry, ThemeType};
