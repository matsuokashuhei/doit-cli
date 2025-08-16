//! Theme system for progress bar rendering
//!
//! This module provides a flexible theme system that allows different
//! visual styles for the progress bar display.

use anyhow::Result;
use chrono::{NaiveDateTime, TimeDelta};
use crossterm::{
    cursor::{Hide, MoveTo},
    queue,
    style::{Color, PrintStyledContent, ResetColor, SetBackgroundColor, Stylize},
    terminal::{size, Clear, ClearType},
};
use std::collections::HashMap;
use std::io::Write;

/// Context data shared across all themes
pub struct RenderContext {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub title: Option<String>,
    pub current_time: NaiveDateTime,
    pub progress: f64,
}

impl RenderContext {
    pub fn new(
        start: NaiveDateTime,
        end: NaiveDateTime,
        title: Option<String>,
        current_time: NaiveDateTime,
        progress: f64,
    ) -> Self {
        Self {
            start,
            end,
            title,
            current_time,
            progress,
        }
    }

    /// Get dynamic format string based on duration
    /// - Within 24 hours: "HH:MM"
    /// - Within 7 days: "mm-dd HH:MM"
    /// - Otherwise: "YYYY-mm-dd"
    pub fn get_time_format(&self) -> &'static str {
        let duration = self.end - self.start;
        let duration_hours = duration.num_hours();

        if duration_hours <= 24 {
            "%H:%M"
        } else if duration.num_days() <= 7 {
            "%m-%d %H:%M"
        } else {
            "%Y-%m-%d"
        }
    }

    /// Format start time according to duration
    pub fn format_start_time(&self) -> String {
        self.start.format(self.get_time_format()).to_string()
    }

    /// Format end time according to duration
    pub fn format_end_time(&self) -> String {
        self.end.format(self.get_time_format()).to_string()
    }

    /// Format start time for retro theme (always full datetime)
    pub fn format_start_time_retro(&self) -> String {
        self.start.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Format end time for retro theme (always full datetime)
    pub fn format_end_time_retro(&self) -> String {
        self.end.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Format total duration according to its length
    /// - Within 1 hour: "m"
    /// - Within 24 hours: "h m"
    /// - Within 7 days: "d h"
    /// - Otherwise: "d"
    pub fn format_total_time(&self) -> String {
        let total_duration = self.end - self.start;
        let total_minutes = total_duration.num_minutes();
        let total_hours = total_duration.num_hours();
        let total_days = total_duration.num_days();

        if total_minutes < 60 {
            // Within 1 hour: show only minutes
            format!("{}m", total_minutes)
        } else if total_hours < 24 {
            // Within 24 hours: show hours and minutes
            let hours = total_hours;
            let minutes = total_minutes % 60;
            if minutes > 0 {
                format!("{}h {}m", hours, minutes)
            } else {
                format!("{}h", hours)
            }
        } else if total_days <= 7 {
            // Within 7 days: show days and hours
            let days = total_days;
            let hours = total_hours % 24;
            if hours > 0 {
                format!("{}d {}h", days, hours)
            } else {
                format!("{}d", days)
            }
        } else if total_days < 365 {
            // Within a year: show weeks and days
            let weeks = total_days / 7;
            let days = total_days % 7;
            if days > 0 {
                format!("{}w {}d", weeks, days)
            } else {
                format!("{}w", weeks)
            }
        } else {
            // 365 days or more: show years and days
            let years = total_days / 365;
            let days = total_days % 365;
            if days > 0 {
                format!("{}y {}d", years, days)
            } else {
                format!("{}y", years)
            }
        }
    }

    pub fn calculate_elapsed_time(&self) -> TimeDelta {
        if self.current_time < self.start {
            // Before the start time: no time elapsed
            TimeDelta::zero()
        } else if self.current_time > self.end {
            // After the end time: full duration elapsed
            self.end - self.start
        } else {
            // During the period: current - start
            self.current_time - self.start
        }
    }

    pub fn calculate_remaining_time(&self) -> TimeDelta {
        if self.current_time < self.start {
            // Before the start time: full duration remaining
            self.end - self.start
        } else if self.current_time > self.end {
            // After the end time: no time remaining
            TimeDelta::zero()
        } else {
            // During the period: end - current
            self.end - self.current_time
        }
    }

    pub fn format_elapsed_time(&self) -> String {
        let elapsed = self.calculate_elapsed_time();
        let minutes = elapsed.num_minutes();
        if minutes < 60 {
            return format!("{minutes} m");
        }
        let hours = elapsed.num_hours();
        if hours < 24 {
            return format!("{} h {} m", hours, minutes % 60);
        }
        let days = elapsed.num_days();
        if days < 365 {
            if days < 3 {
                format!("{} d {} h", days, hours % 24)
            } else {
                format!("{days} d")
            }
        } else {
            // 365 days or more: show years and days
            let years = days / 365;
            let remaining_days = days % 365;
            if remaining_days > 0 {
                format!("{} y {} d", years, remaining_days)
            } else {
                format!("{} y", years)
            }
        }
    }

    pub fn format_remaining_time(&self) -> String {
        let remaining = self.calculate_remaining_time();
        let minutes = remaining.num_minutes();
        if minutes < 60 {
            return format!("{} m", minutes);
        }
        let hours = remaining.num_hours();
        if hours < 24 {
            return format!("{} h {} m", hours, minutes % 60);
        }
        let days = remaining.num_days();
        if days < 365 {
            format!("{} d {} h", days, hours % 24)
        } else {
            // 365 days or more: show years and days
            let years = days / 365;
            let remaining_days = days % 365;
            if remaining_days > 0 {
                format!("{} y {} d", years, remaining_days)
            } else {
                format!("{} y", years)
            }
        }
    }
}

/// Helper function to get current terminal width
pub fn get_terminal_width() -> usize {
    // Use a more conservative width calculation to account for wide characters
    size()
        .map(|(width, _)| (width as usize).saturating_sub(2))
        .unwrap_or(58)
}

/// Theme types enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeType {
    Default,
    Retro,
    Synthwave,
}

/// Base trait for all themes
pub trait Theme: Send + Sync {
    fn name(&self) -> &'static str;
    fn render<W: Write>(&self, context: &RenderContext, w: &mut W) -> Result<u16>;
}

/// Default theme implementation
pub struct DefaultTheme;

impl Theme for DefaultTheme {
    fn name(&self) -> &'static str {
        "default"
    }

    fn render<W: Write>(&self, context: &RenderContext, w: &mut W) -> Result<u16> {
        let bar = DefaultTheme::build_bar(context.progress);

        // Clear screen and reset cursor
        queue!(w, ResetColor, Clear(ClearType::All), Hide)?;

        let mut row = 0;

        // Display title if provided
        if let Some(title) = &context.title {
            queue!(
                w,
                MoveTo(0, row),
                PrintStyledContent(title.to_string().with(Color::Reset))
            )?;
            row += 1;
        }

        // Time range, percentage, and duration info
        let start_time = context.format_start_time();
        let end_time = context.format_end_time();
        let progress_percent = (context.progress * 100.0) as i32;
        let elapsed_time = context.format_elapsed_time();
        let total_time = context.format_total_time();

        let info_line = format!(
            "{} → {}   |   {}%   |   {} / {}",
            start_time, end_time, progress_percent, elapsed_time, total_time
        );
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(info_line.with(Color::Reset))
        )?;
        row += 1;

        // Empty line
        row += 1;

        // Progress bar
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(bar.with(Color::Reset))
        )?;
        row += 1;

        // Empty line
        row += 1;

        // Remaining time
        let remaining_time = context.format_remaining_time();
        let remaining_line = format!("{} remaining", remaining_time);
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(remaining_line.with(Color::Reset))
        )?;

        w.flush()?;
        Ok(row)
    }
}

impl DefaultTheme {
    fn build_bar(progress: f64) -> String {
        let bar_width = get_terminal_width();
        let filled_chars = (progress * bar_width as f64).round() as usize;
        let filled = "█".repeat(filled_chars);
        let empty = "░".repeat(bar_width.saturating_sub(filled_chars));
        format!("{filled}{empty}")
    }
}

/// Retro theme implementation
pub struct RetroTheme;

impl Theme for RetroTheme {
    fn name(&self) -> &'static str {
        "retro"
    }

    fn render<W: Write>(&self, context: &RenderContext, w: &mut W) -> Result<u16> {
        let bar_width = get_terminal_width();
        let bar = RetroTheme::build_retro_bar(context.progress);

        // Clear screen and reset cursor
        queue!(w, ResetColor, Clear(ClearType::All), Hide)?;

        let mut row = 0;

        // Display title with retro styling
        if let Some(title) = &context.title {
            let title_line = format!("[{}] FOCUS SESSION INITIATED", title);
            queue!(
                w,
                MoveTo(0, row),
                PrintStyledContent(title_line.with(Color::Reset).bold())
            )?;
            row += 1;
        }

        // Top border
        let top_border = "=".repeat(bar_width);
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(top_border.with(Color::Reset))
        )?;
        row += 1;

        // Start time
        let start_line = format!("[START]     {}", context.format_start_time_retro());
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(start_line.with(Color::Reset))
        )?;
        row += 1;

        // End time
        let end_line = format!("[END]       {}", context.format_end_time_retro());
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(end_line.with(Color::Reset))
        )?;
        row += 1;

        // Elapsed time
        let elapsed_percent = (context.progress * 100.0) as i32;
        let elapsed_time = context.format_elapsed_time();
        let elapsed_line = format!("[ELAPSED]   {}% | {}", elapsed_percent, elapsed_time);
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(elapsed_line.with(Color::Reset))
        )?;
        row += 1;

        // Remaining time
        let remaining_time = context.format_remaining_time();
        let remaining_line = format!("[REMAINING] {}", remaining_time);
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(remaining_line.with(Color::Reset))
        )?;
        row += 1;

        // Empty line
        row += 1;

        // Progress label
        let progress_label = "[PROGRESS]";
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(progress_label.with(Color::Reset))
        )?;
        row += 1;

        // Progress bar
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(bar.with(Color::Reset))
        )?;
        row += 1;

        // Bottom border
        let bottom_border = "=".repeat(bar_width);
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(bottom_border.with(Color::Reset))
        )?;
        row += 1;

        // Status message
        let status_message = RetroTheme::get_retro_status_message(context.progress);
        let status_line = format!("STATUS: > {}", status_message);
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(status_line.with(Color::Reset).bold())
        )?;
        row += 1;

        // Bottom border
        let bottom_border = "=".repeat(bar_width);
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(bottom_border.with(Color::Reset))
        )?;
        row += 1;

        // Quit instructions
        let quit_text = "(Q) QUIT | (CTRL+C) ABORT";
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(quit_text.with(Color::Reset))
        )?;

        w.flush()?;
        Ok(row)
    }
}

impl RetroTheme {
    fn build_retro_bar(progress: f64) -> String {
        let bar_width = get_terminal_width();
        // Account for the brackets, so inner bar width is bar_width - 2
        let inner_width = bar_width.saturating_sub(2);
        let filled_chars = (progress * inner_width as f64).round() as usize;
        let filled = "█".repeat(filled_chars);
        let empty = "░".repeat(inner_width.saturating_sub(filled_chars));
        format!("[{}]", filled + &empty)
    }

    fn get_retro_status_message(progress: f64) -> &'static str {
        match (progress * 100.0) as i32 {
            0..=10 => "MISSION INITIATED. LOCK AND LOAD, SOLDIER!",
            11..=25 => "ENGAGING TARGET. MAINTAIN FOCUS AND DISCIPLINE.",
            26..=50 => "BATTLE IN PROGRESS. HOLD YOUR POSITION, WARRIOR!",
            51..=75 => "VICTORY IS WITHIN REACH. PUSH FORWARD!",
            76..=90 => "ALMOST THERE, SOLDIER! HOLD YOUR POSITION.",
            91..=99 => "FINAL ASSAULT! BREAK THROUGH THE ENEMY LINES!",
            _ => "MISSION ACCOMPLISHED! EXCELLENT WORK, SOLDIER!",
        }
    }
}

/// Synthwave theme implementation
pub struct SynthwaveTheme;

impl Theme for SynthwaveTheme {
    fn name(&self) -> &'static str {
        "synthwave"
    }

    fn render<W: Write>(&self, context: &RenderContext, w: &mut W) -> Result<u16> {
        let bar_width = get_terminal_width();

        // #35324c
        let bg_color = Color::Rgb {
            r: 59,
            g: 50,
            b: 85,
        };
        //  #30c0b7
        let frame_color = Color::Rgb {
            // r: 48,
            // g: 192,
            // b: 183,
            r: 73,
            g: 128,
            b: 153,
        };
        // #ee227d
        let progress_color = Color::Rgb {
            r: 238,
            g: 34,
            b: 125,
        };
        // #498099
        let text_color = Color::Rgb {
            // r: 73,
            // g: 128,
            // b: 153,
            r: 48,
            g: 192,
            b: 183,
        };
        // #30c0b7
        let title_accent_color = Color::Rgb {
            r: 48,
            g: 192,
            b: 183,
        };
        // #fd8083
        let message_accent_color = Color::Rgb {
            r: 253,
            g: 128,
            b: 131,
        };
        queue!(
            w,
            ResetColor,
            SetBackgroundColor(bg_color),
            Clear(ClearType::All),
            Hide
        )?;

        let mut row = 0;

        // Title with synthwave styling
        if let Some(title) = &context.title {
            let title_line = format!(
                " {} {} {}",
                "═".with(frame_color),
                title.to_uppercase().with(title_accent_color),
                "═".with(frame_color)
            );

            queue!(
                w,
                MoveTo(0, row),
                PrintStyledContent(title_line.with(title_accent_color).on(bg_color).bold())
            )?;
            row += 1;
        }

        // Top border with frame color
        let top_border = format!("╔{}╗", "═".repeat(bar_width.saturating_sub(2)));
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(top_border.with(frame_color).on(bg_color))
        )?;
        row += 1;

        // Progress bar line with time labels
        let start_time = context.format_start_time();
        let end_time = context.format_end_time();

        // Calculate the exact width needed to match border lines
        // Total structure: "║ " + start_time + "  " + bar + "  " + end_time + " ║"
        // Note: Using chars().count() for width - may be inaccurate for multi-byte Unicode
        // characters but adequate for time format strings which use ASCII characters
        let start_time_len = start_time.chars().count();
        let end_time_len = end_time.chars().count();
        let fixed_parts_width = 2 + start_time_len + 2 + 2 + end_time_len + 2; // dynamic calculation based on actual lengths
        let bar_inner_width = bar_width.saturating_sub(fixed_parts_width);

        // Create the progress bar with the correct width
        let bar = SynthwaveTheme::build_synthwave_bar(context.progress, bar_inner_width);

        // The bar is already the correct width, no need to adjust
        let adjusted_bar = bar;

        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent("║ ".with(frame_color).on(bg_color)),
            PrintStyledContent(start_time.with(text_color).on(bg_color)),
            PrintStyledContent("  ".with(Color::Reset).on(bg_color)),
            PrintStyledContent(adjusted_bar.with(progress_color).on(bg_color)),
            PrintStyledContent("  ".with(Color::Reset).on(bg_color)),
            PrintStyledContent(end_time.with(text_color).on(bg_color)),
            PrintStyledContent(" ║".with(frame_color).on(bg_color))
        )?;
        row += 1;

        // Info line
        let progress_percent = (context.progress * 100.0) as i32;
        let elapsed_time = context.format_elapsed_time();
        let remaining_time = context.format_remaining_time();
        let info_text = format!(
            "{}% | {} elapsed | {} remaining",
            progress_percent, elapsed_time, remaining_time
        );
        let fixed_prefix = "║                   "; // 20 characters to align with progress bar start position (║ + 19 spaces)
        let fixed_suffix = "║"; // 1 character

        // Calculate the exact length like the progress bar line does
        let content_length =
            fixed_prefix.chars().count() + info_text.chars().count() + fixed_suffix.chars().count();
        let padding_width = bar_width.saturating_sub(content_length);
        let padding = " ".repeat(padding_width);
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(fixed_prefix.with(frame_color).on(bg_color)),
            PrintStyledContent(info_text.with(text_color).on(bg_color)),
            PrintStyledContent(padding.with(Color::Reset).on(bg_color)),
            PrintStyledContent(fixed_suffix.with(frame_color).on(bg_color))
        )?;
        row += 1;

        // Bottom border with frame color
        let bottom_border = format!("╚{}╝", "═".repeat(bar_width.saturating_sub(2)));
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(bottom_border.with(frame_color).on(bg_color))
        )?;
        row += 1;

        // Synthwave motivation message
        let lightning1 = "⚡";
        let message = " KEEP THE ENERGY FLOWING ";
        let lightning2 = "⚡";
        let full_message = format!("{}{}{}", lightning1, message, lightning2);
        let motivation_padding = " ".repeat((bar_width.saturating_sub(full_message.len())) / 2);
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(motivation_padding.with(Color::Reset).on(bg_color)),
            PrintStyledContent(lightning1.with(Color::Reset).on(bg_color)),
            PrintStyledContent(message.with(message_accent_color).on(bg_color).bold()),
            PrintStyledContent(lightning2.with(Color::Reset).on(bg_color))
        )?;

        w.flush()?;
        Ok(row)
    }
}

impl SynthwaveTheme {
    fn build_synthwave_bar(progress: f64, available_width: usize) -> String {
        let filled_chars = (progress * available_width as f64).round() as usize;
        let filled = "█".repeat(filled_chars);
        let empty = "░".repeat(available_width.saturating_sub(filled_chars));
        format!("{}{}", filled, empty)
    }
}

/// Theme registry for managing all available themes
pub struct ThemeRegistry {
    themes: HashMap<String, ThemeType>,
}

impl ThemeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            themes: HashMap::new(),
        };
        registry.register("default", ThemeType::Default);
        registry.register("retro", ThemeType::Retro);
        registry.register("synthwave", ThemeType::Synthwave);
        registry
    }

    pub fn register(&mut self, name: &str, theme_type: ThemeType) {
        self.themes.insert(name.to_string(), theme_type);
    }

    pub fn get(&self, name: &str) -> Option<ThemeType> {
        self.themes.get(name).copied()
    }

    pub fn list_themes(&self) -> Vec<&str> {
        self.themes.keys().map(|k| k.as_str()).collect()
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    #[test]
    fn test_get_time_format_24h_within() {
        let start =
            NaiveDateTime::parse_from_str("2025-08-16 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end =
            NaiveDateTime::parse_from_str("2025-08-16 18:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let context = RenderContext::new(start, end, None, start, 0.0);

        assert_eq!(context.get_time_format(), "%H:%M");
        assert_eq!(context.format_start_time(), "10:00");
        assert_eq!(context.format_end_time(), "18:00");
    }

    #[test]
    fn test_get_time_format_7d_within() {
        let start =
            NaiveDateTime::parse_from_str("2025-08-16 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end =
            NaiveDateTime::parse_from_str("2025-08-20 18:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let context = RenderContext::new(start, end, None, start, 0.0);

        assert_eq!(context.get_time_format(), "%m-%d %H:%M");
        assert_eq!(context.format_start_time(), "08-16 10:00");
        assert_eq!(context.format_end_time(), "08-20 18:00");
    }

    #[test]
    fn test_get_time_format_over_7d() {
        let start =
            NaiveDateTime::parse_from_str("2025-08-01 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end =
            NaiveDateTime::parse_from_str("2025-09-15 18:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let context = RenderContext::new(start, end, None, start, 0.0);

        assert_eq!(context.get_time_format(), "%Y-%m-%d");
        assert_eq!(context.format_start_time(), "2025-08-01");
        assert_eq!(context.format_end_time(), "2025-09-15");
    }

    #[test]
    fn test_get_time_format_exactly_24h() {
        let start =
            NaiveDateTime::parse_from_str("2025-08-16 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end =
            NaiveDateTime::parse_from_str("2025-08-17 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let context = RenderContext::new(start, end, None, start, 0.0);

        assert_eq!(context.get_time_format(), "%H:%M");
        assert_eq!(context.format_start_time(), "10:00");
        assert_eq!(context.format_end_time(), "10:00");
    }

    #[test]
    fn test_get_time_format_exactly_7d() {
        let start =
            NaiveDateTime::parse_from_str("2025-08-16 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end =
            NaiveDateTime::parse_from_str("2025-08-23 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let context = RenderContext::new(start, end, None, start, 0.0);

        assert_eq!(context.get_time_format(), "%m-%d %H:%M");
        assert_eq!(context.format_start_time(), "08-16 10:00");
        assert_eq!(context.format_end_time(), "08-23 10:00");
    }

    #[test]
    fn test_format_total_time_within_1h() {
        let start =
            NaiveDateTime::parse_from_str("2025-08-16 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end =
            NaiveDateTime::parse_from_str("2025-08-16 10:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let context = RenderContext::new(start, end, None, start, 0.0);

        assert_eq!(context.format_total_time(), "30m");
    }

    #[test]
    fn test_format_total_time_within_24h() {
        let start =
            NaiveDateTime::parse_from_str("2025-08-16 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end =
            NaiveDateTime::parse_from_str("2025-08-16 18:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let context = RenderContext::new(start, end, None, start, 0.0);

        assert_eq!(context.format_total_time(), "8h 30m");
    }

    #[test]
    fn test_format_total_time_within_7d() {
        let start =
            NaiveDateTime::parse_from_str("2025-08-16 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end =
            NaiveDateTime::parse_from_str("2025-08-19 16:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let context = RenderContext::new(start, end, None, start, 0.0);

        assert_eq!(context.format_total_time(), "3d 6h");
    }

    #[test]
    fn test_format_total_time_over_7d() {
        let start =
            NaiveDateTime::parse_from_str("2025-08-01 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end =
            NaiveDateTime::parse_from_str("2025-09-15 18:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let context = RenderContext::new(start, end, None, start, 0.0);

        assert_eq!(context.format_total_time(), "6w 3d");
    }

    #[test]
    fn test_format_total_time_exactly_1h() {
        let start =
            NaiveDateTime::parse_from_str("2025-08-16 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end =
            NaiveDateTime::parse_from_str("2025-08-16 11:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let context = RenderContext::new(start, end, None, start, 0.0);

        assert_eq!(context.format_total_time(), "1h");
    }

    #[test]
    fn test_format_total_time_exactly_weeks() {
        let start =
            NaiveDateTime::parse_from_str("2025-08-01 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end =
            NaiveDateTime::parse_from_str("2025-08-15 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let context = RenderContext::new(start, end, None, start, 0.0);

        assert_eq!(context.format_total_time(), "2w");
    }

    #[test]
    fn test_format_total_time_over_365d() {
        let start =
            NaiveDateTime::parse_from_str("1977-10-31 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end =
            NaiveDateTime::parse_from_str("2057-10-30 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let context = RenderContext::new(start, end, None, start, 0.0);

        assert_eq!(context.format_total_time(), "80y 19d");
    }

    #[test]
    fn test_format_total_time_years_with_days() {
        let start =
            NaiveDateTime::parse_from_str("2023-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end =
            NaiveDateTime::parse_from_str("2025-06-15 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let context = RenderContext::new(start, end, None, start, 0.0);

        assert_eq!(context.format_total_time(), "2y 166d");
    }

    #[test]
    fn test_elapsed_time_before_start() {
        let start =
            NaiveDateTime::parse_from_str("2025-08-16 14:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end =
            NaiveDateTime::parse_from_str("2025-08-16 14:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let current =
            NaiveDateTime::parse_from_str("2025-08-16 13:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let context = RenderContext::new(start, end, None, current, 0.0);

        assert_eq!(context.format_elapsed_time(), "0 m");
        assert_eq!(context.format_remaining_time(), "30 m");
    }

    #[test]
    fn test_elapsed_time_after_end() {
        let start =
            NaiveDateTime::parse_from_str("2025-08-16 14:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end =
            NaiveDateTime::parse_from_str("2025-08-16 14:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let current =
            NaiveDateTime::parse_from_str("2025-08-16 15:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let context = RenderContext::new(start, end, None, current, 0.0);

        assert_eq!(context.format_elapsed_time(), "30 m");
        assert_eq!(context.format_remaining_time(), "0 m");
    }

    #[test]
    fn test_elapsed_time_during_period() {
        let start =
            NaiveDateTime::parse_from_str("2025-08-16 14:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end =
            NaiveDateTime::parse_from_str("2025-08-16 14:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let current =
            NaiveDateTime::parse_from_str("2025-08-16 14:10:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let context = RenderContext::new(start, end, None, current, 0.0);

        assert_eq!(context.format_elapsed_time(), "10 m");
        assert_eq!(context.format_remaining_time(), "20 m");
    }

    #[test]
    fn test_retro_time_format() {
        let start =
            NaiveDateTime::parse_from_str("2025-08-16 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end =
            NaiveDateTime::parse_from_str("2025-08-16 18:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let context = RenderContext::new(start, end, None, start, 0.0);

        // Retro theme should always use full YYYY-mm-dd HH:MM:SS format
        assert_eq!(context.format_start_time_retro(), "2025-08-16 10:00:00");
        assert_eq!(context.format_end_time_retro(), "2025-08-16 18:00:00");

        // While regular format should be dynamic (within 24h = HH:MM)
        assert_eq!(context.format_start_time(), "10:00");
        assert_eq!(context.format_end_time(), "18:00");
    }
}
