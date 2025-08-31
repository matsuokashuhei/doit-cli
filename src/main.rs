use anyhow::Result;
use chrono::Local;
use crossterm::cursor::Hide;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{Clear, ClearType};
use crossterm::{
    cursor::{MoveTo, Show},
    queue,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use doit::timespan::Timespan;
use doit::{
    build_command, Args, DefaultRenderer, HourglassRenderer, RetroRenderer, Style, StyledRenderer,
    SynthwaveRenderer,
};
use std::io::{stdout, Write};
use std::time::Duration;
use tracing::debug;
use tracing_subscriber::EnvFilter;

fn run<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    let command = build_command();
    let args = Args::parse(command.get_matches());
    #[cfg(debug_assertions)]
    debug!(?args);
    let timespan = Timespan::new(args.start.naive_utc(), args.end.naive_utc())?;
    #[cfg(debug_assertions)]
    debug!(?timespan);

    let mut row;
    setup_terminal(w)?;
    loop {
        let current_time = Local::now().naive_local();
        let progress = timespan.progress(current_time);
        debug!(?progress);
        row = match args.style {
            Style::Default => {
                let renderer = DefaultRenderer::new(args.title.clone(), progress);
                renderer.render(w)?
            }
            Style::Hourglass => {
                let renderer = HourglassRenderer::new(args.title.clone(), progress);
                renderer.render(w)?
            }
            Style::Retro => {
                let renderer = RetroRenderer::new(args.title.clone(), progress);
                renderer.render(w)?
            }
            Style::Synthwave => {
                let renderer = SynthwaveRenderer::new(args.title.clone(), progress);
                renderer.render(w)?
            }
        };
        w.flush()?;
        if listen_exit_event(args.interval)? {
            break;
        }
    }
    reset_terminal(w, row)?;
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

fn setup_terminal<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    enable_raw_mode()?;
    queue!(w, Clear(ClearType::All), Hide)?;
    w.flush()?;
    Ok(())
}

fn reset_terminal<W>(w: &mut W, row: u16) -> Result<()>
where
    W: Write,
{
    queue!(w, MoveTo(0, row), Show)?;
    w.flush()?;
    disable_raw_mode()?;
    Ok(())
}

fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::fs::File::create("./doit.log")?)
        .init();
    let mut stdout = stdout();
    run(&mut stdout)
}
