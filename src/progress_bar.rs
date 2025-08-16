//! Progress bar module for the pb CLI tool
//!
//! This module provides progress calculation and rendering functionality
//! for time-based progress visualization with color support.

use crate::theme::{DefaultTheme, RenderContext, RetroTheme, Theme, ThemeRegistry, ThemeType};
use anyhow::Result;
use chrono::{Local, NaiveDateTime, TimeDelta, Timelike};
use std::io::Write;

pub struct ProgressBar {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub title: Option<String>,
    pub theme_registry: ThemeRegistry,
    pub current_theme: String,
}

impl ProgressBar {
    #[allow(clippy::must_use_candidate)]
    pub fn new(
        start: NaiveDateTime,
        end: NaiveDateTime,
        title: Option<String>,
        theme_name: &str,
    ) -> Self {
        ProgressBar {
            start,
            end,
            title,
            theme_registry: ThemeRegistry::new(),
            current_theme: theme_name.to_string(),
        }
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
            let elapsed_duration = current - self.start;
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

    #[allow(clippy::missing_errors_doc)]
    pub fn render<W>(&self, w: &mut W) -> Result<u16>
    where
        W: Write,
    {
        let context = RenderContext::new(
            self.start,
            self.end,
            self.title.clone(),
            Self::current_time(),
            self.calculate_progress_at(None),
        );

        match self
            .theme_registry
            .get(&self.current_theme)
            .unwrap_or(ThemeType::Default)
        {
            ThemeType::Default => {
                let theme = DefaultTheme;
                theme.render(&context, w)
            }
            ThemeType::Retro => {
                let theme = RetroTheme;
                theme.render(&context, w)
            }
        }
    }
}
