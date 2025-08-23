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
    Retro,
    Synthwave,
}

impl Style {
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "retro" => Style::Retro,
            "synthwave" => Style::Synthwave,
            _ => Style::Default,
        }
    }
}

pub trait StyledRenderer {
    fn new(title: Option<String>, progress: Progress) -> Self;
    fn render<W: Write>(&self, w: &mut W) -> Result<u16>;
    fn terminal_width() -> usize {
        size().map_or(80, |(w, _)| w as usize)
    }

    fn render_content<W: Write>(w: &mut W, content: &str, row: u16) -> Result<u16> {
        queue!(
            w,
            MoveTo(0, row),
            Clear(ClearType::CurrentLine),
            PrintStyledContent(content.with(Color::Reset)),
        )?;
        Ok(row + 1)
    }

    fn render_empty_line<W: Write>(w: &mut W, row: u16) -> Result<u16> {
        Self::render_content(w, "", row)?;
        Ok(row + 1)
    }

    fn render_background<W: Write>(w: &mut W, color: Color) -> Result<()> {
        queue!(w, SetBackgroundColor(color))?;
        Ok(())
    }
}
