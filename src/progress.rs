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
    pub fn new(timespan: Timespan, current_time: NaiveDateTime) -> Self {
        let elapsed = if current_time < timespan.from {
            Duration::zero()
        } else if current_time > timespan.to {
            timespan.duration
        } else {
            (current_time - timespan.from).max(Duration::zero())
        };

        let ratio = if timespan.duration.num_seconds() <= 0 {
            1.0
        } else if current_time <= timespan.from {
            0.0
        } else if current_time >= timespan.to {
            1.0
        } else {
            elapsed.num_milliseconds() as f64 / timespan.duration.num_milliseconds() as f64
        };

        let remaining = if current_time > timespan.to {
            Duration::zero()
        } else {
            (timespan.to - current_time).max(Duration::zero())
        };

        Self {
            timespan,
            current_time,
            ratio,
            elapsed,
            remaining,
        }
    }

    pub fn format_remaining(&self) -> String {
        Timespan::format_duration_string(self.remaining)
    }

    pub fn format_elapsed(&self) -> String {
        Timespan::format_duration_string(self.elapsed)
    }

    pub fn is_complete(&self) -> bool {
        self.timespan.has_expired(self.current_time)
    }
}
