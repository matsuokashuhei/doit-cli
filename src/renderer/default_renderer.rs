use crate::{renderer::StyledRenderer, Progress};
use anyhow::Result;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, PrintStyledContent, Stylize},
};
use std::io::Write;

pub struct DefaultRenderer {
    title: Option<String>,
    progress: Progress,
}

impl StyledRenderer for DefaultRenderer {
    fn new(title: Option<String>, progress: Progress) -> Self {
        DefaultRenderer { title, progress }
    }

    fn render<W: Write>(&self, w: &mut W) -> Result<u16> {
        let width = Self::terminal_width();
        let title = self.build_title();
        let row = if let Some(title) = title {
            Self::render_content(w, &title, 0)?
        } else {
            0
        };
        let infromation = self.build_information();
        let row = Self::render_content(w, &infromation, row)?;
        let row = Self::render_empty_line(w, row)?;
        let row = self.render_bar(w, width, row)?;
        let row = Self::render_empty_line(w, row)?;
        let remaining = self.build_remaining();
        let row = Self::render_content(w, &remaining, row)?;
        Ok(row)
    }
}

impl DefaultRenderer {
    fn build_title(&self) -> Option<String> {
        self.title.clone()
    }

    fn build_information(&self) -> String {
        let from = self.progress.timespan.format_from();
        let to = self.progress.timespan.format_to();
        let ratio = format!("{:.0}%", self.progress.ratio * 100.0);
        let space = " ".repeat(3);
        [
            format!("{from} → {to}"),
            ratio,
            format!(
                "{} / {}",
                self.progress.format_elapsed(),
                self.progress.timespan.format_duration()
            ),
        ]
        .join(format!("{space}|{space}").as_str())
    }

    fn render_bar<W: Write>(&self, w: &mut W, width: usize, row: u16) -> Result<u16> {
        let bar_width = width;
        let bar = self.build_bar(bar_width);
        queue!(
            w,
            MoveTo(0, row),
            PrintStyledContent(bar.with(Color::Reset))
        )?;
        Ok(row + 1)
    }

    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_truncation)]
    fn build_bar(&self, width: usize) -> String {
        if self.progress.is_complete() {
            format!("{}{}", "█".repeat(width), "░".repeat(0))
        } else {
            let filled_length = (width as f64 * self.progress.ratio).round() as usize;
            let empty_length = width - filled_length;
            format!("{}{}", "█".repeat(filled_length), "░".repeat(empty_length))
        }
    }

    fn build_remaining(&self) -> String {
        if self.progress.is_complete() {
            "Completed".to_string()
        } else {
            format!("{} remaining", self.progress.format_remaining())
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use crate::{progress, Timespan};

    use super::*;

    #[test]
    fn test_build_bar() {
        let test_cases = vec![
            (0, 100, 0, "░░░░░░░░░░░░░░░░░░░░"),
            (0, 100, 50, "██████████░░░░░░░░░░"),
            (0, 100, 100, "████████████████████"),
        ];
        for (start, end, current, expected) in test_cases {
            let timespan = Timespan::new(
                DateTime::from_timestamp(start, 0).unwrap().naive_utc(),
                DateTime::from_timestamp(end, 0).unwrap().naive_utc(),
            )
            .unwrap();
            let progress = progress::Progress::new(
                timespan,
                DateTime::from_timestamp(current, 0).unwrap().naive_utc(),
            );
            let renderer = DefaultRenderer::new(Some(String::from("Just Do It!")), progress);
            let bar = renderer.build_bar(20);
            assert_eq!(bar, expected);
        }
    }
}
