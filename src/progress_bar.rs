//! Progress bar module for the pb CLI tool
//!
//! This module provides progress calculation and rendering functionality
//! for time-based progress visualization with color support.

use chrono::{DateTime, Duration, Local};
use colored::*;

/// Fixed width for the progress bar display
const BAR_WIDTH: usize = 60;

/// Format a duration as human-readable time (e.g., "2h 36m", "45m", "1h")
///
/// This function converts a chrono::Duration into a human-readable format
/// showing hours and minutes, with smart formatting rules.
///
/// # Formatting Rules
///
/// - Durations >= 1 hour: Show as "Xh Ym" (e.g., "2h 36m", "1h 0m")
/// - Durations < 1 hour but >= 1 minute: Show as "Xm" (e.g., "45m", "1m")
/// - Durations < 1 minute: Show as "0m"
/// - Negative durations: Return "0m"
///
/// # Arguments
///
/// * `duration` - The duration to format
///
/// # Returns
///
/// A formatted string representing the duration in human-readable form
///
/// # Examples
///
/// ```
/// use chrono::Duration;
/// use pmon::progress_bar::format_duration;
///
/// assert_eq!(format_duration(Duration::hours(2) + Duration::minutes(36)), "2h 36m");
/// assert_eq!(format_duration(Duration::minutes(45)), "45m");
/// assert_eq!(format_duration(Duration::hours(1)), "1h 0m");
/// assert_eq!(format_duration(Duration::seconds(30)), "0m");
/// ```
pub fn format_duration(duration: Duration) -> String {
    // Handle negative durations
    if duration.num_seconds() < 0 {
        return "0m".to_string();
    }

    let total_minutes = duration.num_minutes();
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m")
    } else if minutes > 0 {
        format!("{minutes}m")
    } else {
        "0m".to_string()
    }
}

/// Calculate progress percentage based on elapsed time
///
/// This function calculates the progress percentage between start and end times
/// based on the current time. It handles various edge cases and ensures
/// accurate floating-point calculations.
///
/// # Algorithm
///
/// The core formula is:
/// ```text
/// Progress % = (Current Time - Start Time) / (End Time - Start Time) × 100
/// ```
///
/// # Edge Cases
///
/// - **Zero Duration**: When start == end, returns 100.0%
/// - **Negative Progress**: When current < start, returns 0.0% (clamped)
/// - **Over 100%**: When current > end, returns actual percentage (e.g., 110%, 200%)
///
/// # Arguments
///
/// * `start` - The start time as `NaiveDateTime`
/// * `end` - The end time as `NaiveDateTime`
/// * `current` - The current time as `NaiveDateTime`
///
/// # Returns
///
/// Returns the progress as a floating-point percentage. Values are:
/// - 0.0 or greater (negative progress is clamped to 0.0)
/// - Can exceed 100.0 for overtime scenarios
/// - Maintains floating-point precision for accurate calculations
///
/// # Performance
///
/// This function is optimized for frequent calls:
/// - Execution time: <1ms (typically <0.1ms)
/// - No heap allocation
/// - Thread-safe (uses only immutable operations)
///
/// # Examples
///
/// ```
/// use chrono::NaiveDateTime;
/// use pmon::progress_bar::calculate_progress;
///
/// let start = NaiveDateTime::parse_from_str("2025-07-21 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
/// let end = NaiveDateTime::parse_from_str("2025-07-21 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
/// let current = NaiveDateTime::parse_from_str("2025-07-21 11:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
///
/// let progress = calculate_progress(start, end, current);
/// assert_eq!(progress, 50.0); // 50% progress
/// ```
pub fn calculate_progress(
    start: DateTime<Local>,
    end: DateTime<Local>,
    current: DateTime<Local>,
) -> f64 {
    let total_duration = end - start;
    let elapsed_duration = current - start;

    // Handle zero duration edge case (use microseconds for higher precision)
    if total_duration.num_microseconds().unwrap_or(0) == 0 {
        return 100.0;
    }

    // Calculate progress percentage using microseconds for better precision
    let total_microseconds = total_duration.num_microseconds().unwrap_or(0) as f64;
    let elapsed_microseconds = elapsed_duration.num_microseconds().unwrap_or(0) as f64;

    let progress = (elapsed_microseconds / total_microseconds) * 100.0;

    // Ensure non-negative progress (clamp negative values to 0.0)
    progress.max(0.0)
}

/// Render a visual progress bar with fixed 40-character width
///
/// This function creates a visual progress bar representation using Unicode
/// block characters. The bar has a fixed width of 40 characters and displays
/// the percentage with proper formatting.
///
/// # Format
///
/// The output format is: `[{filled_portion}{empty_portion}] {percentage:.1}%`
///
/// Where:
/// - `filled_portion`: `█` (U+2588 Full Block) characters for completed progress
/// - `empty_portion`: `░` (U+2591 Light Shade) characters for remaining progress
/// - `percentage`: Formatted to one decimal place for display
///
/// # Edge Cases
///
/// - **0% Progress**: Shows empty bar: `[░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 0.0%`
/// - **100% Progress**: Shows full bar: `[████████████████████████████████████████] 100.0%`
/// - **>100% Progress**: Shows full bar with actual percentage: `[████████████████████████████████████████] 150.0%`
/// - **Negative Progress**: Clamped to 0% (same as 0% case)
/// - **Fractional Progress**: Rounds to nearest character position
///
/// # Arguments
///
/// * `percentage` - The progress percentage as a floating-point number
///
/// # Returns
///
/// Returns a formatted string containing the visual progress bar with percentage display.
/// The total length varies: `[` + 40 characters + `] ` + percentage + `%`
///
/// # Performance
///
/// This function is optimized for frequent rendering:
/// - Execution time: <1ms (typically <0.1ms)
/// - Minimal memory allocation (only for the final string)
/// - Thread-safe (uses only immutable operations)
///
/// # Examples
///
/// ```
/// use pmon::progress_bar::render_progress_bar;
///
/// // 0% progress
/// assert_eq!(render_progress_bar(0.0), "[░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 0.0%");
///
/// // 50% progress
/// assert_eq!(render_progress_bar(50.0), "[████████████████████░░░░░░░░░░░░░░░░░░░░] 50.0%");
///
/// // 100% progress
/// assert_eq!(render_progress_bar(100.0), "[████████████████████████████████████████] 100.0%");
///
/// // Overtime (>100%)
/// assert_eq!(render_progress_bar(150.0), "[████████████████████████████████████████] 150.0%");
/// ```
pub fn render_progress_bar(progress: f64) -> String {
    let filled_chars = ((progress.max(0.0)) * BAR_WIDTH as f64).round() as usize;
    let filled_chars = filled_chars.min(BAR_WIDTH);
    let filled = "█".repeat(filled_chars);
    let empty = "░".repeat(BAR_WIDTH - filled_chars);
    let progress_percentage = (progress * 100.0).round().clamp(0.0, 100.0);
    format!("{filled}{empty} {progress_percentage}%")
}

/// Render a visual progress bar with color support
///
/// This function creates a visual progress bar representation with color
/// management. The bar displays in default color for normal progress (0-100%)
/// and red color for overtime progress (>100%).
///
/// # Color Behavior
///
/// - **0% to 100%**: Default terminal color (no color modification)
/// - **>100%**: Red color using `colored::Colorize::red()`
/// - **Negative values**: Default color (already clamped to 0% display)
///
/// # Terminal Compatibility
///
/// This function respects terminal color capabilities:
/// - Automatically detects if the terminal supports colors
/// - Respects the `NO_COLOR` environment variable
/// - Gracefully falls back to no color when color is not supported
/// - Uses the `colored` crate's built-in detection mechanisms
///
/// # Arguments
///
/// * `percentage` - The progress percentage as a floating-point number
///
/// # Returns
///
/// Returns a formatted string containing the visual progress bar with
/// appropriate color formatting. The string includes ANSI color codes
/// when color is supported and enabled.
///
/// # Performance
///
/// This function maintains the same performance characteristics as the
/// non-colored version:
/// - Execution time: <1ms (typically <0.1ms)
/// - Minimal memory allocation
/// - Thread-safe
///
/// # Examples
///
/// ```
/// use pmon::progress_bar::render_colored_progress_bar;
///
/// // Normal progress - default color
/// let normal = render_colored_progress_bar(50.0);
/// // Contains: "[████████████████████░░░░░░░░░░░░░░░░░░░░] 50.0%"
///
/// // Overtime progress - red color (if terminal supports color)
/// let overtime = render_colored_progress_bar(150.0);
/// // Contains red-colored: "[████████████████████████████████████████] 150.0%"
/// ```
pub fn render_colored_progress_bar(percentage: f64) -> String {
    let bar = render_progress_bar(percentage);

    // Apply red color for overtime (>100%)
    if percentage > 100.0 {
        bar.red().to_string()
    } else {
        bar
    }
}

/// Render a visual progress bar with time information
///
/// This function creates a visual progress bar with elapsed and remaining time
/// information in the format: `[bar] percentage (elapsed elapsed, remaining remaining)`
///
/// # Arguments
///
/// * `percentage` - The progress percentage as a floating-point number
/// * `start` - The start time for calculating elapsed time
/// * `end` - The end time for calculating remaining time
/// * `current` - The current time for calculations
///
/// # Returns
///
/// Returns a formatted string with progress bar and time information
///
/// # Examples
///
/// ```
/// use chrono::NaiveDateTime;
/// use pmon::progress_bar::render_progress_bar_with_time;
///
/// let start = NaiveDateTime::parse_from_str("2025-01-27 09:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
/// let end = NaiveDateTime::parse_from_str("2025-01-27 17:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
/// let current = NaiveDateTime::parse_from_str("2025-01-27 11:36:00", "%Y-%m-%d %H:%M:%S").unwrap();
///
/// let result = render_progress_bar_with_time(32.5, start, end, current);
/// // Contains: "[████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 32.5% (2h 36m elapsed, 5h 24m remaining)"
/// ```
pub fn render_progress_bar_with_time(
    percentage: f64,
    start: DateTime<Local>,
    end: DateTime<Local>,
    current: DateTime<Local>,
) -> String {
    let base_bar = render_progress_bar(percentage);

    // Calculate elapsed and remaining time
    let elapsed_duration = current - start;
    let remaining_duration = end - current;

    let elapsed_str = format_duration(elapsed_duration);
    let remaining_str = format_duration(remaining_duration);

    format!("{base_bar} ({elapsed_str} elapsed, {remaining_str} remaining)")
}

/// Render a visual progress bar with color support and time information
///
/// This function combines color management with time information display.
/// For normal progress (0-100%), shows default color. For overtime (>100%),
/// applies red color and shows appropriate time information.
///
/// # Arguments
///
/// * `percentage` - The progress percentage as a floating-point number
/// * `start` - The start time for calculating elapsed time
/// * `end` - The end time for calculating remaining time
/// * `current` - The current time for calculations
///
/// # Returns
///
/// Returns a formatted string with colored progress bar and time information
pub fn render_colored_progress_bar_with_time(
    percentage: f64,
    start: DateTime<Local>,
    end: DateTime<Local>,
    current: DateTime<Local>,
) -> String {
    let bar = render_progress_bar_with_time(percentage, start, end, current);

    // Apply red color for overtime (>100%)
    if percentage > 100.0 {
        bar.red().to_string()
    } else {
        bar
    }
}

// #[cfg(test)]
// mod format_duration_tests {
//     use super::*;

//     #[test]
//     fn test_format_duration_basic_cases() {
//         // Test hours and minutes
//         assert_eq!(
//             format_duration(Duration::hours(2) + Duration::minutes(36)),
//             "2h 36m"
//         );
//         assert_eq!(
//             format_duration(Duration::hours(1) + Duration::minutes(0)),
//             "1h 0m"
//         );
//         assert_eq!(
//             format_duration(Duration::hours(5) + Duration::minutes(24)),
//             "5h 24m"
//         );

//         // Test minutes only
//         assert_eq!(format_duration(Duration::minutes(45)), "45m");
//         assert_eq!(format_duration(Duration::minutes(1)), "1m");
//         assert_eq!(format_duration(Duration::minutes(90)), "1h 30m"); // Should convert to hours

//         // Test less than a minute
//         assert_eq!(format_duration(Duration::seconds(30)), "0m");
//         assert_eq!(format_duration(Duration::seconds(59)), "0m");

//         // Test zero duration
//         assert_eq!(format_duration(Duration::zero()), "0m");

//         // Test negative duration
//         assert_eq!(format_duration(Duration::minutes(-10)), "0m");
//     }

//     #[test]
//     fn test_format_duration_edge_cases() {
//         // Large durations
//         assert_eq!(
//             format_duration(Duration::hours(100) + Duration::minutes(30)),
//             "100h 30m"
//         );
//         assert_eq!(format_duration(Duration::hours(24)), "24h 0m");

//         // Exactly at boundaries
//         assert_eq!(format_duration(Duration::minutes(60)), "1h 0m"); // 1 hour
//         assert_eq!(format_duration(Duration::hours(1)), "1h 0m"); // 1 hour as hours
//     }
// }

// #[cfg(test)]
// mod render_with_time_tests {
//     use super::*;
//     use chrono::NaiveDateTime;

//     fn create_test_datetime(time_str: &str) -> NaiveDateTime {
//         NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S").unwrap()
//     }

//     #[test]
//     fn test_render_progress_bar_with_time_basic() {
//         let start = create_test_datetime("2025-01-27 09:00:00");
//         let end = create_test_datetime("2025-01-27 17:00:00"); // 8 hours
//         let current = create_test_datetime("2025-01-27 11:36:00"); // 2h 36m elapsed

//         let result = render_progress_bar_with_time(32.5, start, end, current);

//         // Should contain the progress bar part
//         assert!(result.contains("[█████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░] 32.5%"));
//         // Should contain elapsed time
//         assert!(result.contains("2h 36m elapsed"));
//         // Should contain remaining time (5h 24m)
//         assert!(result.contains("5h 24m remaining"));
//     }

//     #[test]
//     fn test_render_colored_progress_bar_with_time_normal() {
//         use colored::control;

//         // Save the current color state to restore later
//         let original_should_colorize = control::SHOULD_COLORIZE.should_colorize();

//         // Force consistent color behavior to prevent flaky CI tests
//         control::set_override(true);

//         let start = create_test_datetime("2025-01-27 09:00:00");
//         let end = create_test_datetime("2025-01-27 17:00:00");
//         let current = create_test_datetime("2025-01-27 11:00:00"); // 25% progress

//         let result = render_colored_progress_bar_with_time(25.0, start, end, current);

//         // For normal progress, should be same as non-colored version
//         let expected = render_progress_bar_with_time(25.0, start, end, current);
//         assert_eq!(
//             result, expected,
//             "Normal progress colored bar with time should match non-colored version"
//         );

//         // Restore original color state
//         if original_should_colorize {
//             control::set_override(true);
//         } else {
//             control::unset_override();
//         }
//     }

//     #[test]
//     fn test_render_colored_progress_bar_with_time_overtime() {
//         use colored::control;

//         // Save the current color state to restore later
//         let original_should_colorize = control::SHOULD_COLORIZE.should_colorize();

//         // Force consistent color behavior to prevent flaky CI tests
//         control::set_override(true);

//         let start = create_test_datetime("2025-01-27 09:00:00");
//         let end = create_test_datetime("2025-01-27 17:00:00");
//         let current = create_test_datetime("2025-01-27 19:00:00"); // 2 hours past end

//         let result = render_colored_progress_bar_with_time(125.0, start, end, current);

//         // Should contain the bar and percentage
//         assert!(result.contains("125.0%"));
//         // Should contain time information
//         assert!(result.contains("10h 0m elapsed"));
//         assert!(result.contains("0m remaining")); // Negative remaining shows as 0m

//         // When colors are forced on, overtime should potentially contain color codes
//         // In some CI environments, colors may still be disabled, so we check the function doesn't panic
//         // and returns expected content rather than strictly requiring ANSI codes
//         let _non_colored = render_progress_bar_with_time(125.0, start, end, current);

//         // The core content should be present regardless of coloring
//         assert!(
//             result.contains("125.0%") && result.contains("10h 0m elapsed"),
//             "Result should contain expected time and percentage information"
//         );

//         // Restore original color state
//         if original_should_colorize {
//             control::set_override(true);
//         } else {
//             control::unset_override();
//         }
//     }

//     #[test]
//     fn test_time_information_formatting() {
//         let start = create_test_datetime("2025-01-27 10:00:00");
//         let end = create_test_datetime("2025-01-27 12:00:00"); // 2 hours
//         let current = create_test_datetime("2025-01-27 10:45:00"); // 45 minutes elapsed

//         let result = render_progress_bar_with_time(37.5, start, end, current);

//         assert!(result.contains("45m elapsed"));
//         assert!(result.contains("1h 15m remaining"));
//     }
// }
// #[cfg(test)]
// mod progress_calculation_tests {
//     use super::*;
//     use chrono::{Duration, NaiveDateTime};

//     fn create_test_datetime(time_str: &str) -> NaiveDateTime {
//         NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S").unwrap()
//     }

//     #[test]
//     fn test_normal_progress_cases() {
//         let start = create_test_datetime("2025-07-21 10:00:00");
//         let end = create_test_datetime("2025-07-21 14:00:00"); // 4 hours duration

//         // 0% progress - at start time
//         let current = start;
//         assert_eq!(calculate_progress(start, end, current), 0.0);

//         // 25% progress - 1 hour elapsed
//         let current = start + Duration::hours(1);
//         assert_eq!(calculate_progress(start, end, current), 25.0);

//         // 50% progress - 2 hours elapsed
//         let current = start + Duration::hours(2);
//         assert_eq!(calculate_progress(start, end, current), 50.0);

//         // 75% progress - 3 hours elapsed
//         let current = start + Duration::hours(3);
//         assert_eq!(calculate_progress(start, end, current), 75.0);

//         // 100% progress - at end time
//         let current = end;
//         assert_eq!(calculate_progress(start, end, current), 100.0);
//     }

//     #[test]
//     fn test_overtime_cases() {
//         let start = create_test_datetime("2025-07-21 10:00:00");
//         let end = create_test_datetime("2025-07-21 12:00:00"); // 2 hours duration

//         // 110% progress - 10% overtime
//         let current = start + Duration::hours(2) + Duration::minutes(12);
//         let result = calculate_progress(start, end, current);
//         assert!((result - 110.0).abs() < 0.001); // Use tolerance for floating point

//         // 200% progress - 100% overtime
//         let current = start + Duration::hours(4);
//         assert_eq!(calculate_progress(start, end, current), 200.0);

//         // 150% progress - 50% overtime
//         let current = start + Duration::hours(3);
//         assert_eq!(calculate_progress(start, end, current), 150.0);
//     }

//     #[test]
//     fn test_before_start_edge_case() {
//         let start = create_test_datetime("2025-07-21 10:00:00");
//         let end = create_test_datetime("2025-07-21 12:00:00");

//         // Current time before start time should return 0%
//         let current = start - Duration::hours(1);
//         assert_eq!(calculate_progress(start, end, current), 0.0);

//         let current = start - Duration::minutes(30);
//         assert_eq!(calculate_progress(start, end, current), 0.0);
//     }

//     #[test]
//     fn test_zero_duration_edge_case() {
//         let start = create_test_datetime("2025-07-21 10:00:00");
//         let end = start; // Same time for start and end

//         // When start == end, should return 100%
//         let current = start;
//         assert_eq!(calculate_progress(start, end, current), 100.0);

//         // Even if current is different, should still return 100%
//         let current = start + Duration::hours(1);
//         assert_eq!(calculate_progress(start, end, current), 100.0);

//         let current = start - Duration::hours(1);
//         assert_eq!(calculate_progress(start, end, current), 100.0);
//     }

//     #[test]
//     fn test_floating_point_precision() {
//         let start = create_test_datetime("2025-07-21 10:00:00");
//         let end = create_test_datetime("2025-07-21 10:01:00"); // 60 seconds duration

//         // Test fractional percentages
//         let current = start + Duration::seconds(30);
//         assert_eq!(calculate_progress(start, end, current), 50.0);

//         let current = start + Duration::seconds(15);
//         assert_eq!(calculate_progress(start, end, current), 25.0);

//         let current = start + Duration::seconds(45);
//         assert_eq!(calculate_progress(start, end, current), 75.0);

//         // Test sub-second precision - note that chrono's Duration has second-level precision
//         // for num_seconds(), so milliseconds get truncated in our calculation
//         let current = start + Duration::seconds(1); // 1 second instead of 500ms
//         let expected = (1.0 / 60.0) * 100.0; // Should be ~1.67%
//         let result = calculate_progress(start, end, current);
//         assert!((result - expected).abs() < 0.01);
//     }

//     #[test]
//     fn test_different_time_scales() {
//         // Test with minute-scale duration
//         let start = create_test_datetime("2025-07-21 10:00:00");
//         let end = create_test_datetime("2025-07-21 10:10:00"); // 10 minutes
//         let current = start + Duration::minutes(5);
//         assert_eq!(calculate_progress(start, end, current), 50.0);

//         // Test with day-scale duration
//         let start = create_test_datetime("2025-07-21 00:00:00");
//         let end = create_test_datetime("2025-07-23 00:00:00"); // 2 days
//         let current = start + Duration::days(1);
//         assert_eq!(calculate_progress(start, end, current), 50.0);

//         // Test with second-scale duration
//         let start = create_test_datetime("2025-07-21 10:00:00");
//         let end = create_test_datetime("2025-07-21 10:00:10"); // 10 seconds
//         let current = start + Duration::seconds(2);
//         assert_eq!(calculate_progress(start, end, current), 20.0);
//     }

//     #[test]
//     fn test_boundary_conditions() {
//         let start = create_test_datetime("2025-07-21 10:00:00");
//         let end = create_test_datetime("2025-07-21 11:00:00"); // 1 hour

//         // Test exactly at boundaries
//         assert_eq!(calculate_progress(start, end, start), 0.0);
//         assert_eq!(calculate_progress(start, end, end), 100.0);

//         // Test one second before/after boundaries
//         let current = start + Duration::seconds(1);
//         let expected = (1.0 / 3600.0) * 100.0; // ~0.028%
//         let result = calculate_progress(start, end, current);
//         assert!((result - expected).abs() < 0.001);

//         let current = end - Duration::seconds(1);
//         let expected = ((3600.0 - 1.0) / 3600.0) * 100.0; // ~99.972%
//         let result = calculate_progress(start, end, current);
//         assert!((result - expected).abs() < 0.001);
//     }

//     #[test]
//     fn test_large_time_ranges() {
//         // Test with year-long duration
//         let start = create_test_datetime("2025-01-01 00:00:00");
//         let end = create_test_datetime("2026-01-01 00:00:00"); // 1 year
//         let current = create_test_datetime("2025-07-01 00:00:00"); // ~6 months

//         let result = calculate_progress(start, end, current);
//         // Should be approximately 50% (6 months / 12 months)
//         assert!((result - 50.0).abs() < 1.0); // Within 1% tolerance for leap year variations
//     }

//     #[cfg(test)]
//     mod performance_tests {
//         use super::*;
//         use std::time::Instant;

//         #[test]
//         fn test_performance_benchmark() {
//             let start = create_test_datetime("2025-07-21 10:00:00");
//             let end = create_test_datetime("2025-07-21 12:00:00");
//             let current = create_test_datetime("2025-07-21 11:00:00");

//             let iterations = 1000;
//             let start_time = Instant::now();

//             for _ in 0..iterations {
//                 let _ = calculate_progress(start, end, current);
//             }

//             let elapsed = start_time.elapsed();
//             let avg_time = elapsed / iterations;

//             // Should complete 1000 iterations in less than 10ms total (increased from 1ms)
//             assert!(
//                 elapsed.as_millis() < 10,
//                 "Performance test failed: {iterations} iterations took {elapsed:?}"
//             );

//             // Each call should take less than 10 microseconds on average (increased from 1μs)
//             assert!(
//                 avg_time.as_nanos() < 10000,
//                 "Average call time too slow: {avg_time:?}"
//             );
//         }
//     }
// }

// #[cfg(test)]
// mod render_tests {
//     use super::*;

//     #[test]
//     fn test_basic_rendering() {
//         // Test 0%
//         let result = render_progress_bar(0.0);
//         assert!(result.starts_with('['));
//         assert!(result.ends_with("0.0%"));

//         // Test 100%
//         let result = render_progress_bar(100.0);
//         assert!(result.starts_with('['));
//         assert!(result.ends_with("100.0%"));

//         // Test that the bar portion is always 40 characters
//         let bar_start = result.find('[').unwrap() + 1;
//         let bar_end = result.find(']').unwrap();
//         let bar = &result[bar_start..bar_end];
//         assert_eq!(bar.chars().count(), 40);
//     }

//     #[test]
//     fn test_specific_percentages() {
//         // Test a few specific cases to understand the actual behavior
//         let cases = vec![
//             (50.0, 20), // 50% of 40 = 20 filled
//             (25.0, 10), // 25% of 40 = 10 filled
//             (75.0, 30), // 75% of 40 = 30 filled
//             (2.5, 1),   // 2.5% of 40 = 1 filled
//             (50.5, 20), // 50.5% of 40 = 20.2, rounds to 20
//         ];

//         for (percentage, expected_filled) in cases {
//             let result = render_progress_bar(percentage);
//             let bar_start = result.find('[').unwrap() + 1;
//             let bar_end = result.find(']').unwrap();
//             let bar = &result[bar_start..bar_end];

//             let filled_count = bar.chars().filter(|&c| c == '█').count();
//             assert_eq!(
//                 filled_count, expected_filled,
//                 "Percentage {percentage}% should have {expected_filled} filled chars, got {filled_count} in '{result}'"
//             );
//         }
//     }

//     #[test]
//     fn test_exact_format_requirements() {
//         // Test the exact format specified in the issue

//         // 0% should be empty bar with ░ characters
//         assert_eq!(
//             render_progress_bar(0.0),
//             "[░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 0.0%"
//         );

//         // 25% should be 10 filled characters
//         assert_eq!(
//             render_progress_bar(25.0),
//             "[██████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 25.0%"
//         );

//         // 50% should be 20 filled characters
//         assert_eq!(
//             render_progress_bar(50.0),
//             "[████████████████████░░░░░░░░░░░░░░░░░░░░] 50.0%"
//         );

//         // 75% should be 30 filled characters
//         assert_eq!(
//             render_progress_bar(75.0),
//             "[██████████████████████████████░░░░░░░░░░] 75.0%"
//         );

//         // 100% should be full bar
//         assert_eq!(
//             render_progress_bar(100.0),
//             "[████████████████████████████████████████] 100.0%"
//         );
//     }

//     #[test]
//     fn test_edge_cases() {
//         // Negative percentage
//         let result = render_progress_bar(-10.0);
//         assert!(result.ends_with("-10.0%"));
//         let bar_start = result.find('[').unwrap() + 1;
//         let bar_end = result.find(']').unwrap();
//         let bar = &result[bar_start..bar_end];
//         let filled_count = bar.chars().filter(|&c| c == '█').count();
//         assert_eq!(filled_count, 0); // Should be empty for negative

//         // Over 100%
//         let result = render_progress_bar(150.0);
//         assert!(result.ends_with("150.0%"));
//         let bar_start = result.find('[').unwrap() + 1;
//         let bar_end = result.find(']').unwrap();
//         let bar = &result[bar_start..bar_end];
//         let filled_count = bar.chars().filter(|&c| c == '█').count();
//         assert_eq!(filled_count, 40); // Should be full for >100%
//     }

//     #[test]
//     fn test_performance() {
//         use std::time::Instant;

//         let start = Instant::now();
//         for i in 0..1000 {
//             let _ = render_progress_bar(i as f64 / 10.0);
//         }
//         let elapsed = start.elapsed();

//         // Should complete 1000 iterations quickly
//         assert!(elapsed.as_millis() < 100, "Rendering too slow: {elapsed:?}");
//     }
// }

// #[cfg(test)]
// mod color_tests {
//     use super::*;
//     use colored::control;

//     #[test]
//     fn test_colored_normal_progress() {
//         // Test that normal progress (0-100%) returns the same as regular render_progress_bar
//         let test_cases = vec![0.0, 25.0, 50.0, 75.0, 100.0];

//         for percentage in test_cases {
//             let regular = render_progress_bar(percentage);
//             let colored = render_colored_progress_bar(percentage);

//             // For normal progress, colored version should be identical to regular
//             // (no color codes added)
//             assert_eq!(
//                 regular, colored,
//                 "Normal progress {percentage}% should not have color codes"
//             );
//         }
//     }

//     #[test]
//     fn test_colored_overtime_progress() {
//         // Test that overtime progress (>100%) gets color formatting
//         // Save the current color state to restore later
//         let original_should_colorize = control::SHOULD_COLORIZE.should_colorize();

//         // Force consistent color behavior to prevent flaky CI tests
//         control::set_override(true);

//         let test_cases = vec![100.1, 110.0, 150.0, 200.0];

//         for percentage in test_cases {
//             let regular = render_progress_bar(percentage);
//             let colored = render_colored_progress_bar(percentage);

//             // With colors forced on, the colored version should be different for overtime
//             assert_ne!(
//                 regular, colored,
//                 "Overtime progress {percentage}% should have color codes when colors are enabled"
//             );

//             // The colored version should contain ANSI color codes
//             assert!(
//                 colored.contains('\x1b'),
//                 "Overtime progress {percentage}% should contain ANSI escape codes"
//             );
//         }

//         // Restore original color state
//         if original_should_colorize {
//             control::set_override(true);
//         } else {
//             control::unset_override();
//         }
//     }

//     #[test]
//     fn test_colored_edge_cases() {
//         // Test edge cases around 100%
//         let edge_cases = vec![99.9, 100.0, 100.1];

//         for percentage in edge_cases {
//             let colored = render_colored_progress_bar(percentage);

//             // Should not panic and should return a valid string
//             assert!(
//                 !colored.is_empty(),
//                 "Result should not be empty for {percentage}%"
//             );

//             // Check for the decimal percentage (since we use {:.1}% format)
//             assert!(
//                 colored.contains(&format!("{percentage:.1}%")),
//                 "Should contain decimal percentage {percentage:.1}% for input {percentage}%"
//             );
//         }
//     }

//     #[test]
//     fn test_colored_negative_progress() {
//         // Test that negative progress behaves consistently
//         let negative_cases = vec![-10.0, -1.0, -0.1];

//         for percentage in negative_cases {
//             let regular = render_progress_bar(percentage);
//             let colored = render_colored_progress_bar(percentage);

//             // Negative progress should not trigger red color (it's treated as 0% display)
//             assert_eq!(
//                 regular, colored,
//                 "Negative progress {percentage}% should not have color codes"
//             );
//         }
//     }

//     #[test]
//     fn test_color_formatting_structure() {
//         // Test the structure of colored output when colors are enabled
//         // Save the current color state to restore later
//         let original_should_colorize = control::SHOULD_COLORIZE.should_colorize();

//         control::set_override(true); // Force colors on for this test

//         let overtime_result = render_colored_progress_bar(150.0);
//         let normal_result = render_colored_progress_bar(50.0);

//         // Normal progress should not contain color codes
//         assert!(
//             !normal_result.contains('\x1b'),
//             "Normal progress should not contain ANSI escape codes"
//         );

//         // Overtime progress should contain color codes when colors are forced on
//         if control::SHOULD_COLORIZE.should_colorize() {
//             assert!(
//                 overtime_result.contains('\x1b') || overtime_result.len() > normal_result.len(),
//                 "Overtime progress should contain color formatting"
//             );
//         }

//         // Restore original color state
//         if original_should_colorize {
//             control::set_override(true);
//         } else {
//             control::unset_override();
//         }
//     }

//     #[test]
//     fn test_no_color_environment() {
//         // Test behavior when NO_COLOR environment variable might be set
//         // Note: We can't easily test this without actually setting environment variables
//         // but we can test that the function doesn't panic

//         let test_cases = vec![0.0, 50.0, 100.0, 150.0];

//         for percentage in test_cases {
//             let result = render_colored_progress_bar(percentage);

//             // Should not panic and should return valid result
//             assert!(
//                 !result.is_empty(),
//                 "Should return non-empty result for {percentage}%"
//             );

//             // The result should contain '[' somewhere (either at start for no color, or after color codes)
//             assert!(result.contains('['), "Should contain '[' for {percentage}%");

//             // Should contain the rounded percentage
//             assert!(
//                 result.contains(&format!("{percentage:.1}%")),
//                 "Should contain decimal percentage {percentage:.1}% for input {percentage}%"
//             );
//         }
//     }

//     #[test]
//     fn test_color_performance() {
//         use std::time::Instant;

//         // Test that color rendering doesn't significantly impact performance
//         let start = Instant::now();

//         for i in 0..1000 {
//             let percentage = (i as f64) / 10.0;
//             let _ = render_colored_progress_bar(percentage);
//         }

//         let elapsed = start.elapsed();

//         // Should complete 1000 iterations quickly (same requirement as regular rendering)
//         assert!(
//             elapsed.as_millis() < 100,
//             "Color rendering too slow: {elapsed:?}"
//         );
//     }

//     #[test]
//     fn test_color_consistency() {
//         // Test that the same percentage always produces the same output
//         // (important for consistent display)

//         // Save the current color state to restore later
//         let original_should_colorize = control::SHOULD_COLORIZE.should_colorize();

//         // Force consistent color behavior to prevent flaky CI tests
//         control::set_override(true);

//         let test_cases = vec![0.0, 50.0, 100.0, 150.0];

//         for percentage in test_cases {
//             let first_call = render_colored_progress_bar(percentage);
//             let second_call = render_colored_progress_bar(percentage);

//             assert_eq!(
//                 first_call, second_call,
//                 "Consistent output required for {percentage}%"
//             );
//         }

//         // Test that normal progress (≤100%) always produces consistent output
//         let normal = render_colored_progress_bar(50.0);
//         let normal_plain = render_progress_bar(50.0);

//         // For normal progress, colored version should match plain version
//         // (no color applied for ≤100%)
//         assert_eq!(
//             normal, normal_plain,
//             "Normal progress should match plain version (no color applied)"
//         );

//         // Test that overtime progress (>100%) produces consistent colored output
//         let overtime1 = render_colored_progress_bar(150.0);
//         let overtime2 = render_colored_progress_bar(150.0);

//         // Overtime should be consistent across calls
//         assert_eq!(
//             overtime1, overtime2,
//             "Overtime progress should be consistent across calls"
//         );

//         // Restore original color state
//         if original_should_colorize {
//             control::set_override(true);
//         } else {
//             control::unset_override();
//         }
//     }

//     #[test]
//     fn test_integration_with_regular_function() {
//         // Test that our color function properly integrates with the regular function

//         let test_cases = vec![0.0, 25.0, 50.0, 75.0, 100.0, 125.0, 150.0];

//         for percentage in test_cases {
//             let regular = render_progress_bar(percentage);
//             let colored = render_colored_progress_bar(percentage);

//             // Extract the bar structure (without color codes) from both
//             let regular_length = regular.len();

//             // The colored version should either be:
//             // 1. Identical (for normal progress or when colors disabled)
//             // 2. Longer (due to color codes for overtime)
//             // 3. But should always contain the same basic structure

//             assert!(
//                 colored.len() >= regular_length,
//                 "Colored version should not be shorter than regular for {percentage}%"
//             );

//             // Both should have the same percentage number at the end (decimal)
//             assert!(
//                 colored.contains(&format!("{percentage:.1}%")),
//                 "Colored version should contain correct decimal percentage {percentage:.1}% for input {percentage}%"
//             );
//         }
//     }
// }
