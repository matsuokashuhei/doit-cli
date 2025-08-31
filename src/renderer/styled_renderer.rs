use crate::Progress;
use anyhow::Result;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, PrintStyledContent, SetBackgroundColor, Stylize},
    terminal::{size, Clear, ClearType},
};
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Default,
    Hourglass,
    Retro,
    Synthwave,
}

impl Style {
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "hourglass" => Style::Hourglass,
            "retro" => Style::Retro,
            "synthwave" => Style::Synthwave,
            _ => Style::Default,
        }
    }
}

pub trait StyledRenderer {
    #[allow(clippy::missing_errors_doc)]
    fn new(title: Option<String>, progress: Progress) -> Self;

    #[must_use]
    fn render<W: Write>(&self, w: &mut W) -> Result<u16> {
        let row = self.render_content(w);
        queue!(w, Clear(ClearType::FromCursorDown))?;
        row
    }

    #[must_use]
    #[allow(clippy::missing_errors_doc)]
    fn render_content<W: Write>(&self, w: &mut W) -> Result<u16>;

    #[must_use]
    #[allow(clippy::missing_errors_doc)]
    fn terminal_width() -> usize {
        size().map_or(80, |(w, _)| w as usize)
    }

    #[allow(clippy::missing_errors_doc)]
    fn render_content_line<W: Write>(w: &mut W, content: &str, row: u16) -> Result<u16> {
        queue!(
            w,
            MoveTo(0, row),
            Clear(ClearType::CurrentLine),
            PrintStyledContent(content.with(Color::Reset)),
        )?;
        Ok(row + 1)
    }

    #[allow(clippy::missing_errors_doc)]
    fn render_empty_line<W: Write>(w: &mut W, row: u16) -> Result<u16> {
        Self::render_content_line(w, "", row)?;
        Ok(row + 1)
    }

    #[allow(clippy::missing_errors_doc)]
    fn render_background<W: Write>(w: &mut W, color: Color) -> Result<()> {
        queue!(w, SetBackgroundColor(color))?;
        Ok(())
    }

    fn render_xxx<W: Write>(w: &mut W) -> Result<()> {
        queue!(w, Clear(ClearType::FromCursorDown))?;
        Ok(())
    }
}
