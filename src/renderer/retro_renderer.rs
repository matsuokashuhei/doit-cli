use crate::{renderer::StyledRenderer, Progress};
use anyhow::Result;
use std::io::Write;

pub struct RetroRenderer {
    title: Option<String>,
    progress: Progress,
}

impl StyledRenderer for RetroRenderer {
    fn new(title: Option<String>, progress: Progress) -> Self {
        RetroRenderer { title, progress }
    }

    fn render<W: Write>(&self, w: &mut W) -> Result<u16> {
        let width = Self::terminal_width();
        let title = self.build_title();
        let row = if let Some(title) = title {
            Self::render_content(w, &title, 0)?
        } else {
            0
        };
        let divider = Self::buid_divider(width);
        let row = Self::render_content(w, &divider, row)?;
        let from = self.buid_from();
        let row = Self::render_content(w, &from, row)?;
        let to = self.buid_to();
        let row = Self::render_content(w, &to, row)?;
        let elapsed = self.build_elapsed();
        let row = Self::render_content(w, &elapsed, row)?;
        let remaining = self.build_remaining();
        let row = Self::render_content(w, &remaining, row)?;
        let row = Self::render_empty_line(w, row)?;
        let row = Self::render_content(w, "[PROGRESS]", row)?;
        let bar = self.build_bar(width);
        let row = Self::render_content(w, &bar, row)?;
        let row = Self::render_content(w, &divider, row)?;
        let status = self.build_status();
        let row = Self::render_content(w, &status, row)?;
        let row = Self::render_content(w, &divider, row)?;
        let row = Self::render_content(w, "(Q) QUIT | (CTRL+C) ABORT", row)?;
        Ok(row)
    }
}

impl RetroRenderer {
    fn build_title(&self) -> Option<String> {
        if let Some(title) = &self.title {
            let left = "[";
            let right = "]";
            Some(format!(
                "{}{}{} FOCUS SESSION INITIATED",
                left,
                title.to_uppercase(),
                right
            ))
        } else {
            None
        }
    }

    fn buid_divider(width: usize) -> String {
        "=".repeat(width)
    }

    fn buid_from(&self) -> String {
        format!(
            "[START]     {}",
            self.progress
                .timespan
                .format_from_with_string("%Y-%m-%d %H:%M:%S")
        )
    }

    fn buid_to(&self) -> String {
        format!(
            "[END]       {}",
            self.progress
                .timespan
                .format_to_with_string("%Y-%m-%d %H:%M:%S")
        )
    }

    fn build_elapsed(&self) -> String {
        format!(
            "[ELAPSED]   {:.0}% | {}",
            self.progress.ratio * 100.0,
            self.progress.format_elapsed()
        )
    }

    fn build_remaining(&self) -> String {
        format!("[REMAINING] {}", self.progress.format_remaining())
    }

    fn build_bar(&self, width: usize) -> String {
        let lhs = "[";
        let rhs = "]";
        let bar_width = width.saturating_sub(lhs.len() + rhs.len());
        if self.progress.is_complete() {
            format!("{}{}{}{}", lhs, "█".repeat(bar_width), "░".repeat(0), rhs)
        } else {
            let filled_length = (bar_width as f64 * self.progress.ratio).round() as usize;
            let empty_length = bar_width - filled_length;
            format!(
                "{}{}{}{}",
                lhs,
                "█".repeat(filled_length),
                "░".repeat(empty_length),
                rhs
            )
        }
    }

    fn build_status(&self) -> String {
        let status = match (self.progress.ratio * 100.0) as i32 {
            0..=10 => "MISSION INITIATED. LOCK AND LOAD, SOLDIER!",
            11..=25 => "ENGAGING TARGET. MAINTAIN FOCUS AND DISCIPLINE.",
            26..=50 => "BATTLE IN PROGRESS. HOLD YOUR POSITION, WARRIOR!",
            51..=75 => "VICTORY IS WITHIN REACH. PUSH FORWARD!",
            76..=90 => "ALMOST THERE, SOLDIER! HOLD YOUR POSITION.",
            91..=99 => "FINAL ASSAULT! BREAK THROUGH THE ENEMY LINES!",
            _ => "MISSION ACCOMPLISHED! EXCELLENT WORK, SOLDIER!",
        };
        format!("STATUS: > {}", status)
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use crate::{progress, Timespan};

    use super::*;

    // #[test]
    // fn test_build_title() {
    //     let title = "Just Do It!";
    //     let expected = "[JUST DO IT!] FOCUS SESSION INITIATED";
    //     let result = RetroRenderer::build_title(title);
    //     assert_eq!(result, expected);
    // }

    // #[test]
    // fn test_build_bar() {
    //     let test_cases = vec![
    //         (0, 100, 0, "[░░░░░░░░░░░░░░░░░░]"),
    //         (0, 100, 50, "[█████████░░░░░░░░░]"),
    //         (0, 100, 100, "[██████████████████]"),
    //     ];
    //     for (start, end, current, expected) in test_cases {
    //         let timespan = Timespan::new(
    //             DateTime::from_timestamp(start, 0).unwrap().naive_utc(),
    //             DateTime::from_timestamp(end, 0).unwrap().naive_utc(),
    //         )
    //         .unwrap();
    //         let progress = progress::Progress::new(
    //             timespan,
    //             DateTime::from_timestamp(current, 0).unwrap().naive_utc(),
    //         );
    //         let renderer = RetroRenderer::new(progress);
    //         let bar = renderer.build_bar(20);
    //         assert_eq!(bar, expected)
    //     }
    // }
}
