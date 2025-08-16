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

    pub fn calculate_elapsed_time(&self) -> TimeDelta {
        self.current_time - self.start
    }

    pub fn calculate_remaining_time(&self) -> TimeDelta {
        let remaining = self.end - self.current_time;
        if remaining.num_seconds() < 0 {
            TimeDelta::zero()
        } else {
            remaining
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
        if days < 3 {
            format!("{} d {} h", days, hours % 24)
        } else {
            format!("{days} d")
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
        format!("{} d {} h", days, hours % 24)
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
    Cyberpunk,
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
        let start_time = context.start.format("%Y-%m-%d %H:%M");
        let end_time = context.end.format("%Y-%m-%d %H:%M");
        let progress_percent = (context.progress * 100.0) as i32;
        let elapsed_time = context.format_elapsed_time();
        let total_duration = context.end - context.start;
        let total_hours = total_duration.num_hours();
        let total_minutes = total_duration.num_minutes() % 60;
        let total_time = if total_hours > 0 {
            if total_minutes > 0 {
                format!("{}h {}m", total_hours, total_minutes)
            } else {
                format!("{}h", total_hours)
            }
        } else {
            format!("{}m", total_minutes)
        };

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
        let start_line = format!("[START]     {}", context.start.format("%Y-%m-%d %H:%M"));
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(start_line.with(Color::Reset))
        )?;
        row += 1;

        // End time
        let end_line = format!("[END]       {}", context.end.format("%Y-%m-%d %H:%M"));
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

/// Cyberpunk theme implementation
pub struct CyberpunkTheme;

impl Theme for CyberpunkTheme {
    fn name(&self) -> &'static str {
        "cyberpunk"
    }

    fn render<W: Write>(&self, context: &RenderContext, w: &mut W) -> Result<u16> {
        let bar_width = get_terminal_width();

        // Clear screen, set background color, and reset cursor
        let bg_color = Color::Rgb {
            r: 33,
            g: 11,
            b: 75,
        }; // Violet #210B4B
        let frame_color = Color::Rgb {
            r: 106,
            g: 42,
            b: 152,
        }; // Daisy Bush #6A2A98
        let progress_color = Color::Rgb {
            r: 255,
            g: 61,
            b: 148,
        }; // Wild Strawberry #FF3D94
        let text_color = Color::Rgb {
            r: 181,
            g: 48,
            b: 126,
        }; // Medium Red Violet #B5307E
        let title_accent_color = Color::Rgb {
            r: 0,
            g: 206,
            b: 209,
        }; // Bright Cyan #00CED1
        let message_accent_color = Color::Rgb {
            r: 0,
            g: 206,
            b: 209,
        }; // Bright Cyan #00CED1
        queue!(
            w,
            ResetColor,
            SetBackgroundColor(bg_color),
            Clear(ClearType::All),
            Hide
        )?;

        let mut row = 0;

        // Title with cyberpunk styling
        if let Some(title) = &context.title {
            let title_line = format!("[{}]", title.to_uppercase());
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
        let start_time = context.start.format("%Y-%m-%d %H:%M").to_string();
        let end_time = context.end.format("%Y-%m-%d %H:%M").to_string();

        // Calculate the exact width needed to match border lines
        // Total structure: "║ " + start_time + "  " + bar + "  " + end_time + " ║"
        // We know: start_time = "2025-08-12 08:00" (16 chars), end_time same (16 chars)
        let fixed_parts_width = 2 + 16 + 2 + 2 + 16 + 2; // 40 characters total for fixed parts
        let bar_inner_width = bar_width.saturating_sub(fixed_parts_width);

        // Create the progress bar with the correct width
        let bar = CyberpunkTheme::build_cyberpunk_bar(context.progress, bar_inner_width);

        // The bar is already the correct width, no need to adjust
        let adjusted_bar = bar;

        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent("║ ".with(frame_color).on(bg_color)),
            PrintStyledContent(start_time.with(text_color).on(bg_color).bold()),
            PrintStyledContent("  ".with(Color::Reset).on(bg_color)),
            PrintStyledContent(adjusted_bar.with(progress_color).on(bg_color)),
            PrintStyledContent("  ".with(Color::Reset).on(bg_color)),
            PrintStyledContent(end_time.with(text_color).on(bg_color).bold()),
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
            PrintStyledContent(info_text.with(text_color).on(bg_color).bold()),
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

        // Cyberpunk motivation message
        let lightning1 = "⚡";
        let message = " KEEP THE ENERGY FLOWING — CYBER MINDSET ";
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

impl CyberpunkTheme {
    fn build_cyberpunk_bar(progress: f64, available_width: usize) -> String {
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
        registry.register("cyberpunk", ThemeType::Cyberpunk);
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
