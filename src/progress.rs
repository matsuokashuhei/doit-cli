use crate::timespan::Timespan;
use chrono::{Duration, NaiveDateTime};

pub struct Progress {
    pub timespan: Timespan,
    pub current_time: NaiveDateTime,
    pub ratio: f64,
    pub elapsed: Duration,
    pub remaining: Duration,
}

impl Progress {
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn new(timespan: Timespan, current_time: NaiveDateTime) -> Self {
        let elapsed = if current_time < timespan.from {
            Duration::zero()
        } else if current_time > timespan.to {
            timespan.duration
        } else {
            (current_time - timespan.from).max(Duration::zero())
        };

        let remaining = if current_time > timespan.to {
            Duration::zero()
        } else if current_time < timespan.from {
            timespan.duration
        } else {
            (timespan.to - current_time).max(Duration::zero())
        };

        let ratio = {
            let ratio = (elapsed.num_seconds() as f64 / timespan.duration.num_seconds() as f64)
                .clamp(0.0, 1.0);
            f64::from((ratio * 100.0) as i32) / 100.0
        };

        Self {
            timespan,
            current_time,
            ratio,
            elapsed,
            remaining,
        }
    }

    #[must_use]
    pub fn format_remaining(&self) -> String {
        Timespan::format_duration_string(self.remaining)
    }

    #[must_use]
    pub fn format_elapsed(&self) -> String {
        Timespan::format_duration_string(self.elapsed)
    }

    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.timespan.has_expired(self.current_time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_new() {
        let fmt = "%Y-%m-%d %H:%M:%S";
        let from = NaiveDateTime::parse_from_str("2025-09-01 00:00:00", fmt).unwrap();
        let to = NaiveDateTime::parse_from_str("2025-09-01 07:59:59", fmt).unwrap();
        let timespan = Timespan::new(from, to).unwrap();

        // current_time before from
        let progress = Progress::new(timespan, from - Duration::hours(1));
        assert_eq!(progress.ratio, 0.0);
        assert_eq!(progress.elapsed, Duration::zero());
        assert_eq!(progress.remaining, timespan.duration);

        // current_time at from
        let progress = Progress::new(timespan, from);
        assert_eq!(progress.ratio, 0.0);
        assert_eq!(progress.elapsed, Duration::zero());
        assert_eq!(progress.remaining, timespan.duration);

        // current_time between from and to
        let progress = Progress::new(
            timespan,
            from + Duration::minutes(4) + Duration::seconds(48),
        );
        assert_eq!(progress.ratio, 0.01);
        assert_eq!(
            progress.elapsed,
            Duration::minutes(4) + Duration::seconds(48)
        );
        assert_eq!(
            progress.remaining,
            Duration::hours(7) + Duration::minutes(55) + Duration::seconds(11),
        );

        let progress = Progress::new(timespan, from + Duration::minutes(24));
        assert_eq!(progress.ratio, 0.05);
        assert_eq!(progress.elapsed, Duration::minutes(24));
        assert_eq!(
            progress.remaining,
            Duration::hours(7) + Duration::minutes(35) + Duration::seconds(59),
        );

        // current_time between from and to
        let progress = Progress::new(timespan, from + Duration::hours(4));
        assert_eq!(progress.ratio, 0.5);
        assert_eq!(progress.elapsed, Duration::hours(4));
        assert_eq!(
            progress.remaining,
            Duration::hours(3) + Duration::minutes(59) + Duration::seconds(59),
        );

        // current_time at to
        let progress = Progress::new(timespan, to);
        assert_eq!(progress.ratio, 1.0);
        assert_eq!(progress.elapsed, timespan.duration);
        assert_eq!(progress.remaining, Duration::zero());

        // current_time after to
        let progress = Progress::new(timespan, to + Duration::hours(1));
        assert_eq!(progress.ratio, 1.0);
        assert_eq!(progress.elapsed, timespan.duration);
        assert_eq!(progress.remaining, Duration::zero());
    }
}
