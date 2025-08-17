// use anyhow::{Ok, Result};
use anyhow::Result;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::{
    cursor::{MoveTo, Show},
    queue,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use doit::{build_command, Args, ProgressBar};
use std::io::{stdout, Write};
use std::time::Duration;

fn run<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    let command = build_command();
    let args = Args::parse(command.get_matches());
    let progress_bar = ProgressBar::new(
        args.from.naive_utc(),
        args.to.naive_utc(),
        args.title,
        &args.style,
    );

    let mut row;
    enable_raw_mode()?;
    loop {
        row = progress_bar.render(w)?;
        if listen_exit_event(60)? {
            break;
        }
    }
    reset_terminal(w, row)?;
    disable_raw_mode()?;
    Ok(())
}

#[allow(clippy::match_same_arms)]
fn listen_exit_event(timeout: u64) -> Result<bool> {
    if poll(Duration::from_secs(timeout))? {
        match read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::CONTROL,
                state: _,
            }) => {
                return Ok(true);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                kind: KeyEventKind::Press,
                modifiers: _,
                state: _,
            }) => {
                return Ok(true);
            }
            _ => {}
        }
    }
    Ok(false)
}

fn reset_terminal<W>(w: &mut W, row: u16) -> Result<()>
where
    W: Write,
{
    queue!(w, MoveTo(0, row), Show)?;
    w.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    let mut stdout = stdout();
    run(&mut stdout)
}
