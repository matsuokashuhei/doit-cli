use anyhow::{Error, Result};
use chrono::{Local, NaiveDateTime, Timelike};
use thiserror::Error;

pub struct Progress {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
}

#[derive(Error, Debug)]
pub enum ProgressError {
    #[error("Start time must be before or equal to end time")]
    StartAfterEnd,
}

impl Progress {
    pub fn new(start: NaiveDateTime, end: NaiveDateTime) -> Result<Self> {
        if start > end {
            Err(Error::from(ProgressError::StartAfterEnd).into())
        } else {
            print!("Progress created: {} to {}", start, end);
            Ok(Progress { start, end })
        }
    }

    pub fn calculate_progress_at(&self, current: Option<NaiveDateTime>) -> f64 {
        if let Some(current) = current {
            print!("Calculating progress at: {}", current);
            let total_duration = self.end - self.start;
            let elapsed_duration = current - self.start;
            if total_duration.num_milliseconds() == 0 {
                return 1.0;
            }
            let progress = elapsed_duration.num_milliseconds() as f64
                / total_duration.num_milliseconds() as f64;

            progress.max(0.0)
        } else {
            let local = Local::now().with_nanosecond(0).unwrap();
            self.calculate_progress_at(Some(local.naive_utc()))
        }
    }
}

// ...existing code...
