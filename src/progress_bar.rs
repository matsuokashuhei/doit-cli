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
    terminal::{Clear, ClearType},
};
use std::io::Write;

/// Fixed width for the progress bar display
const BAR_WIDTH: usize = 60;

pub struct ProgressBar {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
}

impl ProgressBar {
    pub fn new(start: NaiveDateTime, end: NaiveDateTime) -> Self {
        ProgressBar { start, end }
    }

    fn current_time(&self) -> NaiveDateTime {
        Local::now().naive_local().with_nanosecond(0).unwrap()
    }

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
            self.calculate_progress_at(Some(self.current_time()))
        }
    }

    fn calculate_elapsed_time(&self, current: Option<NaiveDateTime>) -> TimeDelta {
        if let Some(current) = current {
            current - self.start
        } else {
            self.calculate_elapsed_time(Some(self.current_time()))
        }
    }

    fn format_start_time(&self) -> String {
        let label = "Start:";
        let value = self.start.format("%Y-%m-%d %H:%M:%S").to_string();
        self.format_verbose_line(label, &value)
    }

    fn format_end_time(&self) -> String {
        let label = "End:";
        let value = self.end.format("%Y-%m-%d %H:%M:%S").to_string();
        self.format_verbose_line(label, &value)
    }

    fn format_progress(&self, current: NaiveDateTime) -> String {
        let progress = self.calculate_progress_at(Some(current)) * 100.0;
        format!("{:.0} %", progress)
    }

    fn format_progress_and_elapsed(&self) -> String {
        let current_time = self.current_time();
        let label = "Elapsed:";
        let value = format!(
            "{} | {}",
            self.format_progress(current_time),
            self.format_elapsed_time(current_time)
        );
        self.format_verbose_line(label, &value)
    }

    fn format_verbose_line(&self, label: &str, value: &str) -> String {
        let spaces = " ".repeat(BAR_WIDTH - label.len() - value.len());
        format!("{label}{spaces}{value}")
    }

    fn format_elapsed_time(&self, current: NaiveDateTime) -> String {
        let elapsed = self.calculate_elapsed_time(Some(current));
        let minutes = elapsed.num_minutes();
        if minutes < 60 {
            return format!("{} m", minutes);
        }
        let hours = elapsed.num_hours();
        if hours < 24 {
            return format!("{} h {} m", hours, minutes % 60);
        }
        let days = elapsed.num_days();
        if days < 3 {
            format!("{} d {} h", days, hours % 24)
        } else {
            format!("{} d", days)
        }
    }

    fn build_bar(progress: f64) -> String {
        let filled_chars = (progress * BAR_WIDTH as f64).round() as usize;
        let filled = "█".repeat(filled_chars);
        let empty = "░".repeat(BAR_WIDTH - filled_chars);
        format!("{filled}{empty}")
    }

    pub fn render<W>(&self, w: &mut W) -> Result<()>
    where
        W: Write,
    {
        let progress = self.calculate_progress_at(None);
        let bar = Self::build_bar(progress);
        queue!(
            w,
            ResetColor,
            Clear(ClearType::All),
            Hide,
            MoveTo(0, 0),
            PrintStyledContent(bar.clone().with(Color::Reset)),
            MoveTo(0, 1),
            PrintStyledContent(bar.clone().with(Color::Reset)),
            // MoveTo(0, 2),
            // PrintStyledContent(bar.clone().with(Color::Reset)),
            MoveTo(0, 3),
            PrintStyledContent(self.format_start_time().with(Color::Reset)),
            MoveTo(0, 4),
            PrintStyledContent(self.format_end_time().with(Color::Reset)),
            MoveTo(0, 5),
            // PrintStyledContent(format!("Current: {}", self.current_time()).with(Color::Reset)),
            // MoveTo(0, 6),
            PrintStyledContent(self.format_progress_and_elapsed().with(Color::Reset)),
            MoveTo(0, 7),
            PrintStyledContent(format!("(Quit: q or Ctrl+C)",).with(Color::Reset)),
        )?;
        w.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
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
            let progress_bar = ProgressBar::new(start, end);
            assert_eq!(
                progress_bar.calculate_progress_at(Some(current)),
                progress,
                "Failed for start: {}, end: {}, current: {}",
                start,
                end,
                current
            );
        }
    }

    #[test]
    fn test_build_bar() {
        let test_cases = vec![
            (0.0, "░".repeat(BAR_WIDTH)),
            (1.0, "█".repeat(BAR_WIDTH)),
            (0.5, "█".repeat(BAR_WIDTH / 2) + &"░".repeat(BAR_WIDTH / 2)),
        ];
        for (progress, expected) in test_cases {
            assert_eq!(ProgressBar::build_bar(progress), expected);
        }
    }
}
