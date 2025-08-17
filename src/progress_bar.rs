//! Progress bar module for the pb CLI tool
//!
//! This module provides progress calculation and rendering functionality
//! for time-based progress visualization with color support.

use crate::style::{
    DefaultStyle, RenderContext, RetroStyle, Style, StyleRegistry, StyleType, SynthwaveStyle,
};
use anyhow::Result;
use chrono::{Local, NaiveDateTime, Timelike};
use std::io::Write;

pub struct ProgressBar {
    pub from: NaiveDateTime,
    pub to: NaiveDateTime,
    pub title: Option<String>,
    pub style_registry: StyleRegistry,
    pub current_style: String,
}

impl ProgressBar {
    #[allow(clippy::must_use_candidate)]
    pub fn new(
        from: NaiveDateTime,
        to: NaiveDateTime,
        title: Option<String>,
        style_name: &str,
    ) -> Self {
        ProgressBar {
            from,
            to,
            title,
            style_registry: StyleRegistry::new(),
            current_style: style_name.to_string(),
        }
    }

    fn current_time() -> NaiveDateTime {
        Local::now().naive_local().with_nanosecond(0).unwrap()
    }

    #[allow(clippy::cast_precision_loss)]
    fn calculate_progress_at(&self, current: Option<NaiveDateTime>) -> f64 {
        if let Some(current) = current {
            let total_duration = self.to - self.from;
            if total_duration.num_seconds() == 0 {
                return 1.0;
            }
            let elapsed_duration = current - self.from;
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
            self.from,
            self.to,
            self.title.clone(),
            Self::current_time(),
            self.calculate_progress_at(None),
        );

        match self
            .style_registry
            .get(&self.current_style)
            .unwrap_or(StyleType::Default)
        {
            StyleType::Default => {
                let style = DefaultStyle;
                style.render(&context, w)
            }
            StyleType::Retro => {
                let style = RetroStyle;
                style.render(&context, w)
            }
            StyleType::Synthwave => {
                let style = SynthwaveStyle;
                style.render(&context, w)
            }
        }
    }
}
