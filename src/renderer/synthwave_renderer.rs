use anyhow::Result;
use crossterm::style::{Color, Stylize};
use std::io::Write;

use crate::{renderer::StyledRenderer, Progress};

const SPACE: char = ' ';
const VERTICAL_BORDER: char = '║';
const HORIZONTAL_BORDER: char = '═';
const BACKGROUND_COLOR: Color = Color::Rgb {
    r: 59,
    g: 50,
    b: 85,
};

const TEXT_COLOR: Color = Color::Rgb {
    r: 48,
    g: 192,
    b: 183,
};

const BORDER_COLOR: Color = Color::Rgb {
    r: 73,
    g: 128,
    b: 153,
};

const BAR_COLOR: Color = Color::Rgb {
    r: 238,
    g: 34,
    b: 125,
};

const ACCENT_COLOR: Color = Color::Rgb {
    r: 253,
    g: 128,
    b: 131,
};

pub struct SynthwaveRenderer {
    title: Option<String>,
    progress: Progress,
}

impl StyledRenderer for SynthwaveRenderer {
    fn new(title: Option<String>, progress: Progress) -> Self {
        SynthwaveRenderer { title, progress }
    }

    fn render<W: Write>(&self, w: &mut W) -> Result<u16> {
        let width = Self::terminal_width();
        Self::render_background(w, BACKGROUND_COLOR)?;
        let title = self.build_title();
        let row = if let Some(title) = title {
            Self::render_content(w, &title, 0)?
        } else {
            0
        };
        let top_border = self.build_top_border(width);
        let row = Self::render_content(w, &top_border, row)?;
        let bar = self.build_bar(width);
        let row = Self::render_content(w, &bar, row)?;
        let progress = self.build_progress(width);
        let row = Self::render_content(w, &progress, row)?;
        let bottom_border = self.build_bottom_border(width);
        let row = Self::render_content(w, &bottom_border, row)?;
        let message = self.build_message(width);
        let row = Self::render_content(w, &message, row)?;
        Ok(row)
    }
}

impl SynthwaveRenderer {
    fn build_title(&self) -> Option<String> {
        if let Some(title) = &self.title {
            let content = format!(
                "{}{}{}{}{}",
                HORIZONTAL_BORDER.with(BORDER_COLOR).on(BACKGROUND_COLOR),
                SPACE.on(BACKGROUND_COLOR),
                title
                    .to_uppercase()
                    .bold()
                    .with(TEXT_COLOR)
                    .on(BACKGROUND_COLOR),
                SPACE.on(BACKGROUND_COLOR),
                HORIZONTAL_BORDER.with(BORDER_COLOR).on(BACKGROUND_COLOR)
            );
            Some(content)
        } else {
            None
        }
    }

    fn build_top_border(&self, width: usize) -> String {
        let lhs = '╔';
        let rhs = '╗';
        format!(
            "{}{}{}",
            lhs.to_string().with(BORDER_COLOR).on(BACKGROUND_COLOR),
            HORIZONTAL_BORDER
                .to_string()
                .repeat(width.saturating_sub(lhs.len_utf16() + rhs.len_utf16()))
                .with(BORDER_COLOR)
                .on(BACKGROUND_COLOR),
            rhs.to_string().with(BORDER_COLOR).on(BACKGROUND_COLOR),
        )
    }

    fn build_bottom_border(&self, width: usize) -> String {
        let lhs = '╚';
        let rhs = '╝';
        format!(
            "{}{}{}",
            lhs.to_string().with(BORDER_COLOR).on(BACKGROUND_COLOR),
            HORIZONTAL_BORDER
                .to_string()
                .repeat(width.saturating_sub(lhs.len_utf16() + rhs.len_utf16()))
                .with(BORDER_COLOR)
                .on(BACKGROUND_COLOR),
            rhs.to_string().with(BORDER_COLOR).on(BACKGROUND_COLOR),
        )
    }

    fn build_bar(&self, width: usize) -> String {
        let bar_width = self.bar_width(width);
        let filled_length = (bar_width as f64 * self.progress.ratio).round() as usize;
        let empty_length = bar_width - filled_length;
        format!(
            "{}{}{}{}{}{}{}{}{}{}{}{}",
            VERTICAL_BORDER
                .to_string()
                .with(BORDER_COLOR)
                .on(BACKGROUND_COLOR),
            SPACE.on(BACKGROUND_COLOR),
            self.progress
                .timespan
                .format_from()
                .with(TEXT_COLOR)
                .on(BACKGROUND_COLOR),
            SPACE.on(BACKGROUND_COLOR),
            SPACE.on(BACKGROUND_COLOR),
            "█"
                .repeat(filled_length)
                .with(BAR_COLOR)
                .on(BACKGROUND_COLOR),
            "░"
                .repeat(empty_length)
                .with(BAR_COLOR)
                .on(BACKGROUND_COLOR),
            SPACE.on(BACKGROUND_COLOR),
            SPACE.on(BACKGROUND_COLOR),
            self.progress
                .timespan
                .format_to()
                .with(TEXT_COLOR)
                .on(BACKGROUND_COLOR),
            SPACE.on(BACKGROUND_COLOR),
            VERTICAL_BORDER
                .to_string()
                .with(BORDER_COLOR)
                .on(BACKGROUND_COLOR),
        )
    }

    fn bar_width(&self, width: usize) -> usize {
        width
            .saturating_sub(VERTICAL_BORDER.len_utf16())
            .saturating_sub(SPACE.len_utf16())
            .saturating_sub(self.progress.timespan.format_from().len())
            .saturating_sub(SPACE.len_utf16())
            .saturating_sub(SPACE.len_utf16())
            .saturating_sub(SPACE.len_utf16())
            .saturating_sub(SPACE.len_utf16())
            .saturating_sub(self.progress.timespan.format_to().len())
            .saturating_sub(SPACE.len_utf16())
            .saturating_sub(VERTICAL_BORDER.len_utf16())
    }

    fn build_progress(&self, width: usize) -> String {
        let left_space = SPACE.len_utf16()
            + self.progress.timespan.format_from().len()
            + SPACE.len_utf16()
            + SPACE.len_utf16();
        let progress = format!(
            "{:.0}% | {} elapsed | {} remaining",
            self.progress.ratio * 100.0,
            self.progress.format_elapsed(),
            self.progress.timespan.format_duration()
        );
        let rigtht_space = width
            .saturating_sub(VERTICAL_BORDER.len_utf16())
            .saturating_sub(left_space)
            .saturating_sub(progress.len())
            .saturating_sub(VERTICAL_BORDER.len_utf16());
        format!(
            "{}{}{}{}{}",
            VERTICAL_BORDER
                .to_string()
                .with(BORDER_COLOR)
                .on(BACKGROUND_COLOR),
            SPACE.to_string().repeat(left_space).on(BACKGROUND_COLOR),
            progress.with(TEXT_COLOR).on(BACKGROUND_COLOR),
            SPACE.to_string().repeat(rigtht_space).on(BACKGROUND_COLOR),
            VERTICAL_BORDER
                .to_string()
                .with(BORDER_COLOR)
                .on(BACKGROUND_COLOR),
        )
    }

    fn build_message(&self, width: usize) -> String {
        let (symbol_left, message, symbol_right) = if self.progress.is_complete() {
            ('✔', "COMPLETED︎", '✔')
        } else {
            ('⚡', "KEEP THE ENERGY FLOWING", '⚡')
        };
        let message_len = symbol_left.len_utf16()
            + SPACE.len_utf16()
            + message.len()
            + SPACE.len_utf16()
            + symbol_right.len_utf16();
        let left_padding = width.saturating_sub(message_len) / 2;
        let right_padding = width
            .saturating_sub(left_padding)
            .saturating_sub(message_len);
        format!(
            "{}{}{}{}{}{}{}",
            SPACE.to_string().repeat(left_padding).on(BACKGROUND_COLOR),
            symbol_left.with(ACCENT_COLOR).on(BACKGROUND_COLOR),
            SPACE.on(BACKGROUND_COLOR),
            message.with(ACCENT_COLOR).on(BACKGROUND_COLOR),
            SPACE.on(BACKGROUND_COLOR),
            symbol_right.with(ACCENT_COLOR).on(BACKGROUND_COLOR),
            SPACE.to_string().repeat(right_padding).on(BACKGROUND_COLOR)
        )
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;

    use crate::{progress, Timespan};

    use super::*;

    // #[test]
    // fn test_build_title() {
    //     let title = "Just Do It!";
    //     let expected = format!("{} {} {}", "═".bold(), "JUST DO IT!", "═".bold());
    //     let result = SynthwaveRenderer::build_title(title);
    //     assert_eq!(result, expected);
    // }

    // #[test]
    // fn test_build_bar() {
    //     let test_cases = vec![(
    //         NaiveDateTime::parse_from_str("2025-08-16 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
    //         NaiveDateTime::parse_from_str("2025-08-16 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
    //         NaiveDateTime::parse_from_str("2025-08-16 11:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
    //         "║ 10:00 ████████████░░░░░░░░░░░░ 12:00 ║",
    //     )];
    //     for (from, to, current, expected) in test_cases {
    //         let timespan = Timespan::new(from, to).unwrap();
    //         let progress = progress::Progress::new(timespan, current);
    //         let renderer = SynthwaveRenderer::new(progress);
    //         let bar = renderer.build_bar(40);
    //         assert_eq!(bar, expected)
    //     }
    // }
}
