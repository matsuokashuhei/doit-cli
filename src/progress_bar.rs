//! Progress bar module for the pb CLI tool
//!
//! This module provides progress calculation and rendering functionality
//! for time-based progress visualization with color support.

use anyhow::Result;
use chrono::{Local, NaiveDateTime, TimeDelta};
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
    /// Create a new ProgressBar instance with start and end times
    ///
    /// # Arguments
    ///
    /// * `start` - The start time as `NaiveDateTime`
    /// * `end` - The end time as `NaiveDateTime`
    ///
    /// # Returns
    ///
    /// A new `ProgressBar` instance
    pub fn new(start: NaiveDateTime, end: NaiveDateTime) -> Self {
        ProgressBar { start, end }
    }

    fn current_time(&self) -> NaiveDateTime {
        Local::now().naive_utc()
    }

    fn calculate_progress_at(&self, current: Option<NaiveDateTime>) -> f64 {
        if let Some(current) = current {
            let total_duration = self.end - self.start;
            let elapsed_duration = self.calculate_elapsed_time(Some(current));
            if total_duration.num_milliseconds() == 0 {
                return 1.0;
            }
            let progress = elapsed_duration.num_milliseconds() as f64
                / total_duration.num_milliseconds() as f64;
            progress.max(0.0)
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

    fn render_elapsed_time(&self, current: Option<NaiveDateTime>) -> String {
        let elapsed = self.calculate_elapsed_time(current);
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
            PrintStyledContent(format!("Start:   {}", self.start).with(Color::Reset)),
            MoveTo(0, 4),
            PrintStyledContent(format!("End:     {}", self.end).with(Color::Reset)),
            MoveTo(0, 5),
            PrintStyledContent(
                format!(
                    "Elapsed: {:.0} % | {}",
                    progress * 100.0,
                    self.render_elapsed_time(None)
                )
                .with(Color::Reset)
            ),
            MoveTo(0, 7),
            PrintStyledContent(format!("(Quit: q, ESC, or Ctrl+C)",).with(Color::Reset)),
        )?;
        w.flush()?;
        Ok(())
    }
}
