//! The loop: draw when something changed, poll for keys, drain the source.
//!
//! Fullscreen uses the alternate screen; `Options::inline` uses an inline
//! viewport of that many lines instead, so the picker can live under a command
//! it was invoked from (docs/PLAN.md, 08).

use std::io::{self, Stdout};
use std::sync::mpsc::Receiver;
use std::time::Instant;

use crossterm::event::{self, Event};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, ExecutableCommand};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::{Terminal, TerminalOptions, Viewport};
use tuipalette_core::components::Item;
use tuipalette_core::{Outcome, Picker};

use crate::input;

/// How long the loop sleeps when nothing is happening. Short enough that
/// streamed items appear immediately, long enough not to spin.
const POLL: std::time::Duration = std::time::Duration::from_millis(30);

#[derive(Debug, Clone)]
pub struct Options {
    /// Lines to take at the bottom of the terminal; `None` is fullscreen.
    pub inline: Option<u16>,
    /// When set, the phases are printed to stderr as they finish (item 12).
    pub timing: Option<Instant>,
}

pub enum Exit {
    Accepted(String),
    Cancelled,
}

pub fn run(picker: &mut Picker, source: Receiver<Item>, options: &Options) -> io::Result<Exit> {
    let viewport = match options.inline {
        Some(height) => Viewport::Inline(height.max(3)),
        None => Viewport::Fullscreen,
    };
    let fullscreen = matches!(viewport, Viewport::Fullscreen);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    if fullscreen {
        stdout.execute(EnterAlternateScreen)?;
    }
    stdout.execute(cursor::Hide)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::with_options(backend, TerminalOptions { viewport }) {
        Ok(terminal) => terminal,
        Err(error) => {
            disable_raw_mode().ok();
            return Err(error);
        }
    };
    if let Some(start) = options.timing {
        eprintln!("phase terminal-init {}", millis(start));
    }

    let outcome = loop_until_decided(&mut terminal, picker, &source, options)?;

    terminal.show_cursor().ok();
    if fullscreen {
        terminal.backend_mut().execute(LeaveAlternateScreen).ok();
    }
    disable_raw_mode().ok();
    Ok(outcome)
}

fn loop_until_decided(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    picker: &mut Picker,
    source: &Receiver<Item>,
    options: &Options,
) -> io::Result<Exit> {
    let mut dirty = true;
    let mut first = true;
    loop {
        if dirty {
            terminal.draw(|frame| {
                let area: Rect = frame.area();
                picker.render(area, frame.buffer_mut());
            })?;
            dirty = false;
            if first {
                if let Some(start) = options.timing {
                    eprintln!("phase first-frame {}", millis(start));
                }
                first = false;
            }
        }
        if event::poll(POLL)? {
            match event::read()? {
                Event::Key(raw) => {
                    if std::env::var_os("TUIPALETTE_DEBUG_KEYS").is_some() {
                        eprintln!("key {raw:?} -> {:?}", input::translate(raw));
                    }
                    let Some(key) = input::translate(raw) else {
                        continue;
                    };
                    match picker.on_key(key) {
                        Outcome::Quit => return Ok(Exit::Cancelled),
                        Outcome::Invoke { value, .. } => {
                            return Ok(Exit::Accepted(value.unwrap_or_default()));
                        }
                        Outcome::Continue => dirty = true,
                    }
                }
                Event::Resize(_, _) => dirty = true,
                _ => {}
            }
            continue;
        }
        if picker.drain(source) {
            dirty = true;
        }
    }
}

fn millis(start: Instant) -> String {
    format!("{:.1}ms", start.elapsed().as_secs_f64() * 1000.0)
}
