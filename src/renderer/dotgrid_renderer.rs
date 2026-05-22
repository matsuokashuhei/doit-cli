use crate::{renderer::StyledRenderer, Progress, Timespan};
use anyhow::Result;
use crossterm::style::{Color, Stylize};
use crossterm::terminal::size;
use std::io::Write;

const DOT_FILLED: char = '•';
const DOT_EMPTY: char = '•';
const GRID_CELL_WIDTH: usize = 3;

const DOT_FILLED_COLOR: Color = Color::Rgb {
    r: 238,
    g: 238,
    b: 238,
};
const DOT_EMPTY_COLOR: Color = Color::Rgb {
    r: 54,
    g: 54,
    b: 54,
};
const TEXT_COLOR: Color = Color::Rgb {
    r: 245,
    g: 245,
    b: 245,
};
const MUTED_TEXT_COLOR: Color = Color::Rgb {
    r: 52,
    g: 52,
    b: 52,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PanelSize {
    width: usize,
    cols: usize,
    rows: usize,
}

pub struct DotgridRenderer {
    title: Option<String>,
    progress: Progress,
}

impl StyledRenderer for DotgridRenderer {
    fn new(title: Option<String>, progress: Progress) -> Self {
        DotgridRenderer { title, progress }
    }

    fn render_content<W: Write>(&self, w: &mut W) -> Result<u16> {
        let (terminal_width, terminal_height) = Self::terminal_size();
        let size = Self::panel_size(terminal_width, terminal_height);

        let mut row = 0;
        for line in self.build_panel(size) {
            row = Self::render_content_line(w, &line, row)?;
        }

        Ok(row)
    }
}

impl DotgridRenderer {
    fn terminal_size() -> (usize, usize) {
        size().map_or((80, 24), |(width, height)| {
            (usize::from(width), usize::from(height))
        })
    }

    fn panel_size(terminal_width: usize, terminal_height: usize) -> PanelSize {
        let width = terminal_width;
        let cols = width.max(1).div_ceil(GRID_CELL_WIDTH).max(1);
        let rows = terminal_height.saturating_sub(1).max(1);

        PanelSize { width, cols, rows }
    }

    fn build_panel(&self, size: PanelSize) -> Vec<String> {
        let mut lines = Vec::with_capacity(size.rows + 1);
        for grid_row in self.build_grid_rows(size.cols, size.rows) {
            lines.push(Self::content_line(size.width, &grid_row));
        }
        lines.push(self.build_footer(size.width));
        lines
    }

    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    fn filled_dot_count(&self, cols: usize, rows: usize) -> usize {
        let total = cols.saturating_mul(rows);
        if self.progress.is_complete() {
            total
        } else {
            (total as f64 * self.progress.ratio.clamp(0.0, 1.0)).round() as usize
        }
    }

    fn build_grid_rows(&self, cols: usize, rows: usize) -> Vec<String> {
        let filled = self.filled_dot_count(cols, rows);
        let mut lines = Vec::with_capacity(rows);

        for row in 0..rows {
            let mut line = String::new();
            for col in 0..cols {
                if col > 0 {
                    line.push_str(&Self::spaces(GRID_CELL_WIDTH - 1));
                }
                let index = row * cols + col;
                let dot = if index < filled {
                    DOT_FILLED.with(DOT_FILLED_COLOR)
                } else {
                    DOT_EMPTY.with(DOT_EMPTY_COLOR)
                };
                line.push_str(&dot.to_string());
            }
            lines.push(line);
        }

        lines
    }

    fn build_footer(&self, width: usize) -> String {
        let content_width = width;
        let plain_left = self.bottom_left_label();
        let right_plain = self.remaining_label();
        let left = self.bottom_left_label().with(TEXT_COLOR).to_string();
        let remaining = self.remaining_value_label().with(TEXT_COLOR).to_string();
        let suffix = " left".with(MUTED_TEXT_COLOR).to_string();
        let content = if content_width >= plain_left.len() + right_plain.len() {
            let middle = content_width.saturating_sub(plain_left.len() + right_plain.len());
            format!("{left}{}{remaining}{suffix}", Self::spaces(middle))
        } else if content_width >= right_plain.len() {
            let left_padding = content_width.saturating_sub(right_plain.len());
            format!("{}{remaining}{suffix}", Self::spaces(left_padding))
        } else if content_width >= self.remaining_value_label().len() {
            let left_padding = content_width.saturating_sub(self.remaining_value_label().len());
            format!("{}{remaining}", Self::spaces(left_padding))
        } else {
            String::new()
        };

        Self::content_line(width, &content)
    }

    fn bottom_left_label(&self) -> String {
        self.title.clone().unwrap_or_else(|| {
            let days = self.progress.timespan.duration.num_days();
            if (300..=370).contains(&days) {
                self.progress.timespan.format_from_with_string("%Y")
            } else {
                self.progress.timespan.format_from_with_string("%Y-%m-%d")
            }
        })
    }

    fn remaining_label(&self) -> String {
        format!("{} left", self.remaining_value_label())
    }

    fn remaining_value_label(&self) -> String {
        Timespan::format_duration_string(self.progress.remaining)
    }

    fn content_line(width: usize, content: &str) -> String {
        let plain_len = Self::strip_ansi(content).chars().count();
        let padding = width.saturating_sub(plain_len);
        format!("{}{}", content, Self::spaces(padding))
    }

    fn spaces(width: usize) -> String {
        " ".repeat(width)
    }

    fn strip_ansi(content: &str) -> String {
        let mut plain = String::with_capacity(content.len());
        let mut chars = content.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\u{1b}' && chars.peek() == Some(&'[') {
                chars.next();
                for seq in chars.by_ref() {
                    if seq.is_ascii_alphabetic() {
                        break;
                    }
                }
            } else {
                plain.push(ch);
            }
        }

        plain
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;

    use crate::{progress, renderer::StyledRenderer, Timespan};

    use super::*;

    fn renderer_at(current: &str) -> DotgridRenderer {
        let fmt = "%Y-%m-%d %H:%M:%S";
        let from = NaiveDateTime::parse_from_str("2026-01-01 00:00:00", fmt).unwrap();
        let to = NaiveDateTime::parse_from_str("2027-01-01 00:00:00", fmt).unwrap();
        let current = NaiveDateTime::parse_from_str(current, fmt).unwrap();
        let timespan = Timespan::new(from, to).unwrap();
        let progress = progress::Progress::new(timespan, current);
        DotgridRenderer::new(None, progress)
    }

    fn renderer_for(from: &str, to: &str, current: &str) -> DotgridRenderer {
        let fmt = "%Y-%m-%d %H:%M:%S";
        let from = NaiveDateTime::parse_from_str(from, fmt).unwrap();
        let to = NaiveDateTime::parse_from_str(to, fmt).unwrap();
        let current = NaiveDateTime::parse_from_str(current, fmt).unwrap();
        let timespan = Timespan::new(from, to).unwrap();
        let progress = progress::Progress::new(timespan, current);
        DotgridRenderer::new(None, progress)
    }

    #[test]
    fn dot_counts_follow_progress_ratio() {
        let test_cases = [
            ("2026-01-01 00:00:00", 20, 10, 0),
            ("2026-07-02 12:00:00", 20, 10, 100),
            ("2027-01-01 00:00:00", 20, 10, 200),
        ];

        for (current, cols, rows, expected) in test_cases {
            let renderer = renderer_at(current);
            assert_eq!(renderer.filled_dot_count(cols, rows), expected);
        }
    }

    #[test]
    fn bottom_metadata_uses_title_and_days_left() {
        let mut renderer = renderer_at("2026-01-18 00:00:00");
        renderer.title = Some("Year Focus".to_string());

        assert_eq!(renderer.bottom_left_label(), "Year Focus");
        assert_eq!(renderer.remaining_label(), "348d left");
    }

    #[test]
    fn remaining_label_uses_dynamic_duration_units() {
        let less_than_year = renderer_at("2026-01-18 00:00:00");
        let more_than_year = renderer_for(
            "2026-01-01 00:00:00",
            "2028-01-01 00:00:00",
            "2026-01-01 00:00:00",
        );

        assert_eq!(less_than_year.remaining_label(), "348d left");
        assert_eq!(more_than_year.remaining_label(), "2y left");
    }

    #[test]
    fn panel_size_follows_terminal_size() {
        let size = DotgridRenderer::panel_size(120, 40);

        assert_eq!(size.width, 120);
        assert_eq!(size.rows, 39);
        assert!(size.cols >= 1);
    }

    #[test]
    fn panel_contains_no_border_characters() {
        let renderer = renderer_at("2026-01-18 00:00:00");
        let panel = renderer.build_panel(DotgridRenderer::panel_size(24, 6));
        let output = panel.join("\n");

        assert!(!output.contains('╭'));
        assert!(!output.contains('╮'));
        assert!(!output.contains('╰'));
        assert!(!output.contains('╯'));
        assert!(!output.contains('│'));
        assert!(!output.contains('─'));
    }

    #[test]
    fn narrow_terminal_sizing_stays_nonzero() {
        let size = DotgridRenderer::panel_size(2, 2);

        assert_eq!(size.width, 2);
        assert_eq!(size.cols, 1);
        assert_eq!(size.rows, 1);
    }
}
