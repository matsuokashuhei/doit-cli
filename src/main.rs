// use anyhow::{Ok, Result};
use anyhow::Result;
use clap::Parser;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    queue,
    style::{Color, PrintStyledContent, ResetColor, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use pmon::{progress::Progress, render_progress_bar, Args};
use std::io::{stdout, Write};
use std::time::Duration;

fn run<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    let args = Args::parse();

    enable_raw_mode()?;

    loop {
        let progress = Progress::new(args.start.naive_utc(), args.end.naive_utc())?;
        let bar = render_progress_bar(progress.calculate_progress_at(None));
        queue!(
            w,
            ResetColor,
            Clear(ClearType::All),
            Hide,
            MoveTo(0, 0),
            PrintStyledContent(bar.with(Color::Reset))
        )?;
        w.flush()?;
        if poll(Duration::from_secs(args.interval))? {
            match read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::CONTROL,
                    state: _,
                }) => {
                    break;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    kind: KeyEventKind::Press,
                    modifiers: _,
                    state: _,
                }) => {
                    break;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Esc,
                    kind: KeyEventKind::Press,
                    modifiers: _,
                    state: _,
                }) => {
                    break;
                }
                _ => {}
            };
        }
    }
    cleanup(w)?;
    disable_raw_mode()?;
    Ok(())
}

fn cleanup<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    queue!(w, ResetColor, Clear(ClearType::All), Show, MoveTo(0, 0),)?;
    w.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let mut stdout = stdout();
    run(&mut stdout)
}
