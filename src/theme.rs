//! Theme system for progress bar rendering
//!
//! This module provides a flexible theme system that allows different
//! visual styles for the progress bar display.

use anyhow::Result;
use chrono::{NaiveDateTime, TimeDelta};
use crossterm::{
    cursor::{Hide, MoveTo},
    queue,
    style::{Color, PrintStyledContent, ResetColor, Stylize},
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
    pub bar_width: usize,
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
            bar_width: Self::bar_width(),
        }
    }

    fn bar_width() -> usize {
        // Use a more conservative width calculation to account for wide characters
        size()
            .map(|(width, _)| (width as usize).saturating_sub(2))
            .unwrap_or(58)
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
            return format!("{:02}m", minutes);
        }
        let hours = remaining.num_hours();
        if hours < 24 {
            return format!("{:02}h {:02}m", hours, minutes % 60);
        }
        let days = remaining.num_days();
        format!("{:02}d {:02}h", days, hours % 24)
    }
}

/// Theme types enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeType {
    Default,
    Retro,
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
        let bar = self.build_bar(context.progress);
        let bar_width = context.bar_width;

        // Clear screen and reset cursor
        queue!(w, ResetColor, Clear(ClearType::All), Hide)?;

        let mut row = 0;

        // Display title if provided
        if let Some(title) = &context.title {
            queue!(
                w,
                MoveTo(0, row),
                PrintStyledContent(title.to_string().with(Color::Reset).bold())
            )?;
            row += 1;
            let top_border = "━".repeat(bar_width).to_string();
            queue!(
                w,
                MoveTo(0, row),
                PrintStyledContent(top_border.with(Color::Reset))
            )?;
            row += 1;
        }

        // Display progress bar
        for _ in 0..3 {
            queue!(
                w,
                MoveTo(0, row),
                PrintStyledContent(bar.clone().with(Color::Reset))
            )?;
            row += 1;
        }

        // Draw bordered box
        let top_border = format!("┏{}┓", "━".repeat(bar_width.saturating_sub(2)));
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(top_border.with(Color::Reset))
        )?;
        row += 1;

        // Start time row
        let start_line = format!("┃ {} ┃", self.format_start_time_for_box(context));
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(start_line.with(Color::Reset))
        )?;
        row += 1;

        // End time row
        let end_line = format!("┃ {} ┃", self.format_end_time_for_box(context));
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(end_line.with(Color::Reset))
        )?;
        row += 1;

        // Middle separator
        let middle_border = format!("┠{}┨", "─".repeat(bar_width.saturating_sub(2)));
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(middle_border.with(Color::Reset))
        )?;
        row += 1;

        // Elapsed time row
        let elapsed_line = format!("┃ {} ┃", self.format_progress_and_elapsed_for_box(context));
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(elapsed_line.with(Color::Reset))
        )?;
        row += 1;

        // Bottom border
        let bottom_border = format!("┗{}┛", "━".repeat(bar_width.saturating_sub(2)));
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(bottom_border.with(Color::Reset))
        )?;
        row += 1;

        // Quit instructions (left-aligned, below the box)
        let quit_text = "( Quit: q or Ctrl+c )";
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(quit_text.with(Color::Reset))
        )?;

        w.flush()?;
        Ok(row)
    }
}

impl DefaultTheme {
    fn build_bar(&self, progress: f64) -> String {
        let bar_width = RenderContext::bar_width();
        let filled_chars = (progress * bar_width as f64).round() as usize;
        let filled = "█".repeat(filled_chars);
        let empty = "░".repeat(bar_width.saturating_sub(filled_chars));
        format!("{filled}{empty}")
    }

    fn format_start_time_for_box(&self, context: &RenderContext) -> String {
        let label = "Start:";
        let value = context.start.format("%Y-%m-%d %H:%M:%S").to_string();
        self.format_box_line(label, &value, context.bar_width)
    }

    fn format_end_time_for_box(&self, context: &RenderContext) -> String {
        let label = "End:";
        let value = context.end.format("%Y-%m-%d %H:%M:%S").to_string();
        self.format_box_line(label, &value, context.bar_width)
    }

    fn format_progress_and_elapsed_for_box(&self, context: &RenderContext) -> String {
        let label = "Elapsed:";
        let progress_percent = (context.progress * 100.0) as i32;
        let value = format!(
            "{}% | {}",
            progress_percent,
            context.format_elapsed_time()
        );
        self.format_box_line(label, &value, context.bar_width)
    }

    fn format_box_line(&self, label: &str, value: &str, bar_width: usize) -> String {
        // Account for borders (subtract 4 for "┃ " and " ┃")
        let available_width = bar_width.saturating_sub(4);
        let spaces = " ".repeat(available_width.saturating_sub(label.len() + value.len()));
        format!("{label}{spaces}{value}")
    }
}

/// Retro theme implementation
pub struct RetroTheme;

impl Theme for RetroTheme {
    fn name(&self) -> &'static str {
        "retro"
    }

    fn render<W: Write>(&self, context: &RenderContext, w: &mut W) -> Result<u16> {
        let bar = self.build_retro_bar(context.progress);
        let bar_width = context.bar_width;

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
        let start_line = format!("[START]     {}", context.start.format("%Y-%m-%d %H:%M:%S"));
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(start_line.with(Color::Reset))
        )?;
        row += 1;

        // End time
        let end_line = format!("[END]       {}", context.end.format("%Y-%m-%d %H:%M:%S"));
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
        let status_message = self.get_retro_status_message(context.progress);
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
    fn build_retro_bar(&self, progress: f64) -> String {
        let bar_width = RenderContext::bar_width();
        // Account for the brackets, so inner bar width is bar_width - 2
        let inner_width = bar_width.saturating_sub(2);
        let filled_chars = (progress * inner_width as f64).round() as usize;
        let filled = "█".repeat(filled_chars);
        let empty = "░".repeat(inner_width.saturating_sub(filled_chars));
        format!("[{}]", filled + &empty)
    }

    fn get_retro_status_message(&self, progress: f64) -> &'static str {
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
