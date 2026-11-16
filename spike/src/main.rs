mod action;
mod app;
mod component;
mod spec;

use std::io;
use std::time::Duration;

use ratatui::crossterm::event::{self, Event};
use ratatui::DefaultTerminal;

use app::App;
use spec::UiSpec;

fn main() -> io::Result<()> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "spec.ron".to_string());
    let text = std::fs::read_to_string(&path)?;
    let spec = UiSpec::parse(&text).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut app = App::new(spec);
    let mut terminal = ratatui::init();
    let outcome = run(&mut terminal, &mut app);
    ratatui::restore();
    outcome?;

    match app.take_result() {
        Some(value) => {
            println!("{value}");
            Ok(())
        }
        None => std::process::exit(1),
    }
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    while !app.quit {
        terminal.draw(|frame| app.draw(frame))?;
        if event::poll(Duration::from_millis(200))?
            && let Event::Key(key) = event::read()?
        {
            app.on_key(key);
        }
    }
    Ok(())
}
