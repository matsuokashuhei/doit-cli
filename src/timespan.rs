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
        if from >= to {
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

    #[test]
    fn test_new_if_result_ok() {
        let test_cases = [
            ("2025-09-01 00:00:00", "2025-09-01 00:00:01"),
            ("2025-09-01 00:00:00", "2025-09-01 00:01:00"),
            ("2025-09-01 00:00:00", "2025-09-01 01:00:00"),
            ("2025-09-01 00:00:00", "2025-09-02 00:00:00"),
            ("2025-09-01 00:00:00", "2025-10-01 00:00:00"),
            ("2025-09-01 00:00:00", "2026-09-01 00:00:00"),
        ];
        for (from_str, to_str) in test_cases {
            let from = NaiveDateTime::parse_from_str(from_str, "%Y-%m-%d %H:%M:%S").unwrap();
            let to = NaiveDateTime::parse_from_str(to_str, "%Y-%m-%d %H:%M:%S").unwrap();
            assert!(
                Timespan::new(from, to).is_ok(),
                "from: {from_str}, to: {to_str}"
            );
        }
    }

    #[test]
    fn test_new_if_result_err() {
        let test_cases = [
            ("2025-09-01 00:00:00", "2025-09-01 00:00:00"),
            ("2025-09-01 00:00:01", "2025-09-01 00:00:00"),
            ("2025-09-01 00:01:00", "2025-09-01 00:00:00"),
            ("2025-09-01 01:00:00", "2025-09-01 00:00:00"),
            ("2025-09-02 00:00:00", "2025-09-01 00:00:00"),
            ("2025-10-01 00:00:00", "2025-10-01 00:00:00"),
            ("2026-09-01 00:00:00", "2026-09-01 00:00:00"),
        ];
        for (from_str, to_str) in test_cases {
            let from = NaiveDateTime::parse_from_str(from_str, "%Y-%m-%d %H:%M:%S").unwrap();
            let to = NaiveDateTime::parse_from_str(to_str, "%Y-%m-%d %H:%M:%S").unwrap();
            assert!(
                Timespan::new(from, to).is_err(),
                "from: {from_str}, to: {to_str}"
            );
        }
    }

    #[test]
    fn test_has_expired() {
        let test_cases = [
            (
                "2025-09-01 00:00",
                "2025-09-01 12:00",
                "2025-09-01 00:00",
                false,
            ),
            (
                "2025-09-01 00:00",
                "2025-09-01 12:00",
                "2025-09-01 06:00",
                false,
            ),
            (
                "2025-09-01 00:00",
                "2025-09-01 12:00",
                "2025-09-01 12:00",
                true,
            ),
            (
                "2025-09-01 00:00",
                "2025-09-01 12:00",
                "2025-08-31 00:00",
                false,
            ),
            (
                "2025-09-01 00:00",
                "2025-09-01 12:00",
                "2025-09-01 18:00",
                true,
            ),
        ];
        for (from_str, to_str, current_str, expected) in test_cases {
            let from = NaiveDateTime::parse_from_str(from_str, "%Y-%m-%d %H:%M").unwrap();
            let to = NaiveDateTime::parse_from_str(to_str, "%Y-%m-%d %H:%M").unwrap();
            let current_time =
                NaiveDateTime::parse_from_str(current_str, "%Y-%m-%d %H:%M").unwrap();
            let timespan = Timespan::new(from, to).unwrap();
            assert_eq!(timespan.has_expired(current_time), expected);
        }
    }

    #[test]
    fn test_progress() {
        let from = NaiveDateTime::parse_from_str("2025-09-01 00:00", "%Y-%m-%d %H:%M").unwrap();
        let to = NaiveDateTime::parse_from_str("2025-09-01 07:59", "%Y-%m-%d %H:%M").unwrap();
        let current_time =
            NaiveDateTime::parse_from_str("2025-09-01 03:59", "%Y-%m-%d %H:%M").unwrap();
        let timespan = Timespan::new(from, to).unwrap();
        let progress = timespan.progress(current_time);
        assert_eq!(timespan, progress.timespan);
        assert_eq!(current_time, progress.current_time);
    }

    #[test]
    fn test_format_from() {
        let from = NaiveDateTime::parse_from_str("2025-09-01 00:00", "%Y-%m-%d %H:%M").unwrap();
        let to = NaiveDateTime::parse_from_str("2025-09-01 07:59", "%Y-%m-%d %H:%M").unwrap();
        let timespan = Timespan::new(from, to).unwrap();
        assert_eq!(timespan.format_from(), "00:00");
    }

    #[test]
    fn test_format_to() {
        let fmt = "%Y-%m-%d %H:%M";
        let from = NaiveDateTime::parse_from_str("2025-09-01 00:00", fmt).unwrap();
        let to = NaiveDateTime::parse_from_str("2025-09-01 07:59", fmt).unwrap();
        let timespan = Timespan::new(from, to).unwrap();
        assert_eq!(timespan.format_to(), "07:59");
    }

    #[test]
    fn test_format_from_with_string() {
        let fmt = "%Y-%m-%d %H:%M:%S";
        let from = NaiveDateTime::parse_from_str("2025-09-01 00:00:00", fmt).unwrap();
        let to = NaiveDateTime::parse_from_str("2025-09-01 07:59:59", fmt).unwrap();
        let timespan = Timespan::new(from, to).unwrap();
        assert_eq!(
            timespan.format_from_with_string("%Y-%m-%d %H:%M:%S"),
            "2025-09-01 00:00:00"
        );
        assert_eq!(timespan.format_from_with_string("%H:%M:%S"), "00:00:00");
    }

    #[test]
    fn test_format_duration() {
        let fmt = "%Y-%m-%d %H:%M:%S";
        let from = NaiveDateTime::parse_from_str("2025-09-01 00:00:00", fmt).unwrap();
        let to = NaiveDateTime::parse_from_str("2025-09-01 07:59:59", fmt).unwrap();
        let timespan = Timespan::new(from, to).unwrap();
        assert_eq!(timespan.format_duration(), "7 h 59 m");
    }

    #[test]
    fn test_format_to_with_string() {
        let fmt = "%Y-%m-%d %H:%M:%S";
        let from = NaiveDateTime::parse_from_str("2025-09-01 00:00:00", fmt).unwrap();
        let to = NaiveDateTime::parse_from_str("2025-09-01 07:59:59", fmt).unwrap();
        let timespan = Timespan::new(from, to).unwrap();
        assert_eq!(
            timespan.format_to_with_string("%Y-%m-%d %H:%M:%S"),
            "2025-09-01 07:59:59"
        );
        assert_eq!(timespan.format_to_with_string("%H:%M:%S"), "07:59:59");
    }

    #[test]
    fn test_duration_strings() {
        let test_cases = [
            (Duration::minutes(45), "45 m"),
            (Duration::hours(5), "5 h"),
            (Duration::hours(5) + Duration::minutes(30), "5 h 30 m"),
            (Duration::days(3), "3 d"),
            (Duration::days(3) + Duration::hours(12), "3 d 12 h"),
            (Duration::days(10), "10 d"),
            (Duration::days(30), "30 d"),
            (Duration::days(300), "300 d"),
            (Duration::days(400), "1 y"),
            (Duration::days(800), "2 y"),
        ];
        for (duration, expected) in test_cases {
            assert_eq!(Timespan::format_duration_string(duration), expected);
        }
    }

    #[test]
    fn test_format_hours() {
        let duration = Duration::hours(5);
        assert_eq!(Timespan::format_hours(duration), "5 h");
        let duration = Duration::hours(5) + Duration::minutes(30);
        assert_eq!(Timespan::format_hours(duration), "5 h 30 m");
    }

    #[test]
    fn test_format_days() {
        let duration = Duration::days(3);
        assert_eq!(Timespan::format_days(duration), "3 d");
        let duration = Duration::days(3) + Duration::hours(12);
        assert_eq!(Timespan::format_days(duration), "3 d 12 h");
    }

    #[test]
    fn format_string() {
        let test_cases = [
            ("2025-09-01 00:00", "2025-09-01 12:00", "%H:%M"),
            ("2025-09-01 00:00", "2025-09-01 23:59", "%H:%M"),
            ("2025-09-01 00:00", "2025-09-02 00:00", "%m-%d %H:%M"),
            ("2025-09-01 00:00", "2025-09-03 23:59", "%m-%d %H:%M"),
            ("2025-09-01 00:00", "2025-09-30 23:59", "%Y-%m-%d"),
        ];
        for (from_str, to_str, expected) in test_cases {
            let from = NaiveDateTime::parse_from_str(from_str, "%Y-%m-%d %H:%M").unwrap();
            let to = NaiveDateTime::parse_from_str(to_str, "%Y-%m-%d %H:%M").unwrap();
            let timespan = Timespan::new(from, to).unwrap();
            assert_eq!(timespan.format_string(), expected);
        }
    }
}
