use crate::{error::DoItError, Progress};
use anyhow::{format_err, Result};
use chrono::{Duration, NaiveDateTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timespan {
    pub from: NaiveDateTime,
    pub to: NaiveDateTime,
    pub duration: Duration,
}

impl Timespan {
    #[allow(clippy::missing_errors_doc)]
    pub fn new(from: NaiveDateTime, to: NaiveDateTime) -> Result<Self> {
        if from > to {
            Err(format_err!(DoItError::FromAfterTo { from, to }))
        } else {
            let duration = to - from;
            Ok(Timespan { from, to, duration })
        }
    }

    #[must_use]
    pub fn has_expired(&self, current_time: NaiveDateTime) -> bool {
        current_time >= self.to
    }
}

impl Timespan {
    #[must_use]
    pub fn progress(&self, current_time: NaiveDateTime) -> Progress {
        Progress::new(*self, current_time)
    }

    #[must_use]
    pub fn format_from(&self) -> String {
        self.from.format(self.format_string()).to_string()
    }

    #[must_use]
    pub fn format_from_with_string(&self, string: &str) -> String {
        self.from.format(string).to_string()
    }

    #[must_use]
    pub fn format_to(&self) -> String {
        self.to.format(self.format_string()).to_string()
    }

    #[must_use]
    pub fn format_to_with_string(&self, string: &str) -> String {
        self.to.format(string).to_string()
    }

    #[must_use]
    pub fn format_duration(&self) -> String {
        Self::format_duration_string(self.duration)
    }

    #[must_use]
    pub fn format_duration_string(duration: Duration) -> String {
        let minutes = duration.num_minutes();
        let hours = duration.num_hours();
        let days = duration.num_days();
        if minutes < 60 {
            format!("{minutes} m")
        } else if hours < 24 {
            Self::format_hours(duration)
        } else if days < 7 {
            Self::format_days(duration)
        } else if days < 365 {
            format!("{days} d")
        } else {
            format!("{} y", days / 365)
        }
    }

    fn format_hours(duration: Duration) -> String {
        let hours = duration.num_hours();
        let minutes = duration.num_minutes() % 60;
        if minutes == 0 {
            format!("{hours} h")
        } else {
            format!("{hours} h {minutes} m")
        }
    }

    fn format_days(duration: Duration) -> String {
        let days = duration.num_days();
        let hours = duration.num_hours() % 24;
        if hours == 0 {
            format!("{days} d")
        } else {
            format!("{days} d {hours} h")
        }
    }

    fn format_string(&self) -> &str {
        if self.duration.num_hours() < 24 {
            "%H:%M"
        } else if self.duration.num_weeks() < 3 {
            "%m-%d %H:%M"
        } else {
            "%Y-%m-%d"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    #[test]
    fn test_new_if_result_ok() {
        let from = DateTime::from_timestamp(1_000_000_000, 0)
            .unwrap()
            .naive_utc();
        let to = DateTime::from_timestamp(1_000_000_100, 0)
            .unwrap()
            .naive_utc();
        assert!(Timespan::new(from, to).is_ok());
        let from = DateTime::from_timestamp(1_000_000_100, 0)
            .unwrap()
            .naive_utc();
        let to = DateTime::from_timestamp(1_000_000_100, 0)
            .unwrap()
            .naive_utc();
        assert!(Timespan::new(from, to).is_ok());
    }

    #[test]
    fn test_new_if_result_err() {
        let from = DateTime::from_timestamp(1_000_000_100, 0)
            .unwrap()
            .naive_utc();
        let to = DateTime::from_timestamp(1_000_000_000, 0)
            .unwrap()
            .naive_utc();
        assert!(Timespan::new(from, to).is_err());
    }
}
