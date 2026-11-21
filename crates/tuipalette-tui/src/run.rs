//! The loop: draw when something changed, poll for keys, drain the source.
//!
//! Fullscreen uses the alternate screen; `Options::inline` uses an inline
//! viewport of that many lines instead, so the picker can live under a command
//! it was invoked from (docs/PLAN.md, 08).

use std::io::{self, Stdout};
use std::sync::mpsc::Receiver;
use std::time::Instant;

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, ExecutableCommand};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::{Terminal, TerminalOptions, Viewport};
use tuipalette_core::components::Item;
use tuipalette_core::{Outcome, Picker};

use crate::graphics::{self, Graphics, Protocol};
use crate::input;

/// The picture backend, held for exactly as long as the run: whatever it put
/// up is deleted on every way out, the way `MouseCapture` gives the mouse
/// back — a normal exit, an error, and a panic unwinding through `run`.
struct Pictures(Option<Box<dyn Graphics>>);

impl Drop for Pictures {
    fn drop(&mut self) {
        if let Some(backend) = self.0.as_mut() {
            backend.clear(&mut io::stdout()).ok();
        }
    }
}

/// How long the loop sleeps when nothing is happening. Short enough that
/// streamed items appear immediately, long enough not to spin.
const POLL: std::time::Duration = std::time::Duration::from_millis(30);

#[derive(Debug, Clone)]
pub struct Options {
    /// Lines to take at the bottom of the terminal; `None` is fullscreen.
    pub inline: Option<u16>,
    /// When set, the phases are printed to stderr as they finish (item 12).
    pub timing: Option<Instant>,
    /// Which picture backend to use for `Image` panes (D22).
    pub images: Protocol,
}

impl Default for Options {
    fn default() -> Self {
        Self { inline: None, timing: None, images: Protocol::Auto }
    }
}

pub enum Exit {
    Accepted(String),
    Cancelled,
}

/// Mouse capture, held for exactly as long as this value lives. Capture takes
/// text selection away from the terminal, so it must be given back on every
/// way out: a normal exit, an `?` on an I/O error, and a panic unwinding
/// through `run` — `Drop` covers all three. No profile sets
/// `panic = "abort"`; if one ever does, nothing unwinds and this needs a
/// panic hook (and so does raw mode, which has the same exposure today).
struct MouseCapture;

impl MouseCapture {
    fn enable() -> io::Result<Self> {
        io::stdout().execute(EnableMouseCapture)?;
        Ok(Self)
    }
}

impl Drop for MouseCapture {
    fn drop(&mut self) {
        io::stdout().execute(DisableMouseCapture).ok();
    }
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

    // Only when the document asks (docs/specs/21-mouse.md): a picker that
    // captures the mouse by default breaks select-to-copy for every caller.
    let capture = if picker.session().spec().window.mouse {
        Some(MouseCapture::enable()?)
    } else {
        None
    };
    // Inline runs share the screen with the shell; a picture there would
    // outlive the viewport's rows, so pictures are a fullscreen feature.
    let mut pictures = Pictures(if fullscreen { graphics::detect(options.images) } else { None });
    let outcome = loop_until_decided(&mut terminal, picker, &source, options, &mut pictures);
    drop(pictures);
    drop(capture);
    let outcome = outcome?;

    terminal.show_cursor().ok();
    if fullscreen {
        terminal.backend_mut().execute(LeaveAlternateScreen).ok();
    } else {
        // An inline picker owns the bottom N lines, and ratatui leaves them
        // drawn with the cursor back at their first cell — so the shell's next
        // line lands on top of a half-erased frame. Wipe the region and park
        // the cursor where the next line of output belongs.
        let height = options.inline.unwrap_or_default();
        let top = crossterm::terminal::size()
            .map(|(_, rows)| rows.saturating_sub(height))
            .unwrap_or_default();
        terminal.clear().ok();
        terminal.backend_mut().execute(cursor::MoveTo(0, top)).ok();
    }
    disable_raw_mode().ok();
    Ok(outcome)
}

fn loop_until_decided(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    picker: &mut Picker,
    source: &Receiver<Item>,
    options: &Options,
    pictures: &mut Pictures,
) -> io::Result<Exit> {
    let mut dirty = true;
    let mut first = true;
    // The rect the last frame was drawn in. A mouse event is in terminal
    // cells, and an inline viewport does not start at row 0, so the hit test
    // has to use the very rect the picture was laid out in.
    let mut drawn = Rect::default();
    loop {
        if dirty {
            let mut placed = Ok(());
            terminal.draw(|frame| {
                let area: Rect = frame.area();
                drawn = area;
                picker.render(area, frame.buffer_mut());
                // After the cells, before the flush: a backend may overwrite
                // a fallback with its own cells and write escapes first, so
                // the picture and the frame reach the terminal together.
                if let Some(backend) = pictures.0.as_mut() {
                    let images = picker.images(area);
                    placed = backend.place(&images, frame.buffer_mut(), &mut io::stdout());
                }
            })?;
            placed?;
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
                Event::Mouse(raw) => {
                    let Some(mouse) = input::translate_mouse(raw) else {
                        continue;
                    };
                    match picker.on_mouse(drawn, mouse) {
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
