//! Progress bar module for the pb CLI tool
//!
//! This module provides progress calculation and rendering functionality
//! for time-based progress visualization with color support.

use anyhow::Result;
use chrono::{Local, NaiveDateTime, TimeDelta, Timelike};
use crossterm::{
    cursor::{Hide, MoveTo},
    queue,
    style::{Color, PrintStyledContent, ResetColor, Stylize},
    terminal::{size, Clear, ClearType},
};
use std::io::Write;
use crate::Theme;

pub struct ProgressBar {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub title: Option<String>,
    pub theme: Theme,
}

impl ProgressBar {
    #[allow(clippy::must_use_candidate)]
    pub fn new(start: NaiveDateTime, end: NaiveDateTime, title: Option<String>, theme: Theme) -> Self {
        ProgressBar { start, end, title, theme }
    }

    fn current_time() -> NaiveDateTime {
        Local::now().naive_local().with_nanosecond(0).unwrap()
    }

    #[allow(clippy::cast_precision_loss)]
    fn calculate_progress_at(&self, current: Option<NaiveDateTime>) -> f64 {
        if let Some(current) = current {
            let total_duration = self.end - self.start;
            if total_duration.num_seconds() == 0 {
                return 1.0;
            }
            let elapsed_duration = self.calculate_elapsed_time(Some(current));
            if elapsed_duration > total_duration {
                return 1.0;
            }
            let progress =
                elapsed_duration.num_seconds() as f64 / total_duration.num_seconds() as f64;
            (progress.max(0.0) * 100.0).round() / 100.0
        } else {
            self.calculate_progress_at(Some(Self::current_time()))
        }
    }

    fn calculate_elapsed_time(&self, current: Option<NaiveDateTime>) -> TimeDelta {
        if let Some(current) = current {
            current - self.start
        } else {
            self.calculate_elapsed_time(Some(Self::current_time()))
        }
    }

    fn format_start_time_for_box(&self) -> String {
        let label = "Start:";
        let value = self.start.format("%Y-%m-%d %H:%M:%S").to_string();
        Self::format_box_line(label, &value)
    }

    fn format_end_time_for_box(&self) -> String {
        let label = "End:";
        let value = self.end.format("%Y-%m-%d %H:%M:%S").to_string();
        Self::format_box_line(label, &value)
    }

    fn format_progress(&self, current: NaiveDateTime) -> String {
        let progress = self.calculate_progress_at(Some(current)) * 100.0;
        format!("{progress:.0} %")
    }

    fn format_progress_and_elapsed_for_box(&self) -> String {
        let current_time = Self::current_time();
        let label = "Elapsed:";
        let value = format!(
            "{} | {}",
            self.format_progress(current_time),
            self.format_elapsed_time(current_time)
        );
        Self::format_box_line(label, &value)
    }

    fn format_box_line(label: &str, value: &str) -> String {
        // Account for borders (subtract 4 for "┃ " and " ┃")
        let available_width = Self::bar_width().saturating_sub(4);
        let spaces = " ".repeat(available_width.saturating_sub(label.len() + value.len()));
        format!("{label}{spaces}{value}")
    }

    fn format_elapsed_time(&self, current: NaiveDateTime) -> String {
        let elapsed = self.calculate_elapsed_time(Some(current));
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

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_sign_loss)]
    fn build_bar(progress: f64) -> String {
        let filled_chars = (progress * ProgressBar::bar_width() as f64).round() as usize;
        let filled = "█".repeat(filled_chars);
        let empty = "░".repeat(ProgressBar::bar_width() - filled_chars);
        format!("{filled}{empty}")
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn render<W>(&self, w: &mut W) -> Result<u16>
    where
        W: Write,
    {
        match self.theme {
            Theme::Retro => self.render_retro(w),
            Theme::Default => self.render_default(w),
        }
    }

    #[allow(clippy::missing_errors_doc)]
    fn render_default<W>(&self, w: &mut W) -> Result<u16>
    where
        W: Write,
    {
        let progress = self.calculate_progress_at(None);
        let bar = ProgressBar::build_bar(progress);
        let bar_width = ProgressBar::bar_width();

        // Clear screen and reset cursor
        queue!(w, ResetColor, Clear(ClearType::All), Hide)?;

        let mut row = 0;

        // Display title if provided
        if let Some(title) = &self.title {
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
        let start_line = format!("┃ {} ┃", self.format_start_time_for_box());
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(start_line.with(Color::Reset))
        )?;
        row += 1;

        // End time row
        let end_line = format!("┃ {} ┃", self.format_end_time_for_box());
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
        let elapsed_line = format!("┃ {} ┃", self.format_progress_and_elapsed_for_box());
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

        // Quit instructions (right-aligned, below the box)
        let quit_text = "( Quit: q or Ctrl+c )";
        let quit_padding = " ".repeat(bar_width.saturating_sub(quit_text.len()));
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(format!("{quit_padding}{quit_text}").with(Color::Reset))
        )?;

        w.flush()?;
        Ok(row)
    }

    fn bar_width() -> usize {
        size().map(|(width, _)| width as usize).unwrap_or(60)
    }

    fn calculate_remaining_time(&self, current: NaiveDateTime) -> TimeDelta {
        let remaining = self.end - current;
        if remaining.num_seconds() < 0 {
            TimeDelta::zero()
        } else {
            remaining
        }
    }

    fn format_remaining_time(&self, current: NaiveDateTime) -> String {
        let remaining = self.calculate_remaining_time(current);
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

    fn build_retro_bar(&self, progress: f64) -> String {
        let filled_chars = (progress * ProgressBar::bar_width() as f64).round() as usize;
        let filled = "█".repeat(filled_chars);
        let empty = "░".repeat(ProgressBar::bar_width() - filled_chars);
        format!("[{}]", filled + &empty)
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn render_retro<W>(&self, w: &mut W) -> Result<u16>
    where
        W: Write,
    {
        let current_time = Self::current_time();
        let progress = self.calculate_progress_at(Some(current_time));
        let bar = self.build_retro_bar(progress);
        let bar_width = ProgressBar::bar_width();

        // Clear screen and reset cursor
        queue!(w, ResetColor, Clear(ClearType::All), Hide)?;

        let mut row = 0;

        // Display title with retro styling
        if let Some(title) = &self.title {
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
        let start_line = format!("[START]     {}", self.start.format("%Y-%m-%d %H:%M:%S"));
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(start_line.with(Color::Reset))
        )?;
        row += 1;

        // End time
        let end_line = format!("[END]       {}", self.end.format("%Y-%m-%d %H:%M:%S"));
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(end_line.with(Color::Reset))
        )?;
        row += 1;

        // Elapsed time
        let elapsed_percent = (progress * 100.0) as i32;
        let elapsed_time = self.format_elapsed_time(current_time);
        let elapsed_line = format!("[ELAPSED]   {}% | {}", elapsed_percent, elapsed_time);
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(elapsed_line.with(Color::Reset))
        )?;
        row += 1;

        // Remaining time
        let remaining_time = self.format_remaining_time(current_time);
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
        let status_message = self.get_retro_status_message(progress);
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
        let quit_padding = " ".repeat(bar_width.saturating_sub(quit_text.len()));
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(format!("{quit_padding}{quit_text}").with(Color::Reset))
        )?;

        w.flush()?;
        Ok(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_calculate_progress_at() {
        let test_cases = vec![
            (
                "2025-01-01 00:00:00",
                "2025-01-10 23:59:59",
                "2025-01-01 00:00:00",
                0.0,
            ),
            (
                "2025-01-01 00:00:00",
                "2025-01-10 23:59:59",
                "2025-01-06 00:00:00",
                0.5,
            ),
            (
                "2025-01-01 00:00:00",
                "2025-01-10 23:59:59",
                "2025-01-10 23:59:59",
                1.0,
            ),
            (
                "2025-01-01 00:00:00",
                "2025-01-10 23:59:59",
                "2025-01-11 00:00:00",
                1.0,
            ),
            (
                "2025-01-01 00:00:00",
                "2025-01-10 23:59:59",
                "2025-01-12 00:00:00",
                1.0,
            ),
        ];
        for (start, end, current, progress) in test_cases {
            let start = NaiveDateTime::parse_from_str(start, "%Y-%m-%d %H:%M:%S").unwrap();
            let end = NaiveDateTime::parse_from_str(end, "%Y-%m-%d %H:%M:%S").unwrap();
            let current = NaiveDateTime::parse_from_str(current, "%Y-%m-%d %H:%M:%S").unwrap();
            let progress_bar = ProgressBar::new(start, end, None, Theme::Default);
            assert_eq!(
                progress_bar.calculate_progress_at(Some(current)),
                progress,
                "Failed for start: {start}, end: {end}, current: {current}",
            );
        }
    }

    // #[test]
    // fn test_build_bar() {
    //     let test_cases = vec![
    //         (0.0, "░".repeat(60)),
    //         (1.0, "█".repeat(60)),
    //         (0.5, "█".repeat(30) + &"░".repeat(30)),
    //     ];
    //     for (progress, expected) in test_cases {
    //         let progress_bar = ProgressBar::new(start, end);
    //         assert_eq!(progress_bar.build_bar(progress), expected);
    //     }
    // }

    #[test]
    fn test_get_retro_status_message() {
        let start = NaiveDateTime::parse_from_str("2025-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end = NaiveDateTime::parse_from_str("2025-01-10 23:59:59", "%Y-%m-%d %H:%M:%S").unwrap();
        let progress_bar = ProgressBar::new(start, end, None, Theme::Retro);

        assert_eq!(
            progress_bar.get_retro_status_message(0.05),
            "MISSION INITIATED. LOCK AND LOAD, SOLDIER!"
        );
        assert_eq!(
            progress_bar.get_retro_status_message(0.15),
            "ENGAGING TARGET. MAINTAIN FOCUS AND DISCIPLINE."
        );
        assert_eq!(
            progress_bar.get_retro_status_message(0.35),
            "BATTLE IN PROGRESS. HOLD YOUR POSITION, WARRIOR!"
        );
        assert_eq!(
            progress_bar.get_retro_status_message(0.65),
            "VICTORY IS WITHIN REACH. PUSH FORWARD!"
        );
        assert_eq!(
            progress_bar.get_retro_status_message(0.85),
            "ALMOST THERE, SOLDIER! HOLD YOUR POSITION."
        );
        assert_eq!(
            progress_bar.get_retro_status_message(0.95),
            "FINAL ASSAULT! BREAK THROUGH THE ENEMY LINES!"
        );
        assert_eq!(
            progress_bar.get_retro_status_message(1.0),
            "MISSION ACCOMPLISHED! EXCELLENT WORK, SOLDIER!"
        );
    }

    #[test]
    fn test_format_remaining_time() {
        let start = NaiveDateTime::parse_from_str("2025-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end = NaiveDateTime::parse_from_str("2025-01-01 01:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let progress_bar = ProgressBar::new(start, end, None, Theme::Retro);

        let current = NaiveDateTime::parse_from_str("2025-01-01 00:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
        assert_eq!(progress_bar.format_remaining_time(current), "01h 00m");

        let current = NaiveDateTime::parse_from_str("2025-01-01 01:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        assert_eq!(progress_bar.format_remaining_time(current), "30m");

        let current = NaiveDateTime::parse_from_str("2025-01-01 01:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
        assert_eq!(progress_bar.format_remaining_time(current), "00m");
    }
}
