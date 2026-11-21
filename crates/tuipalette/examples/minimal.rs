//! The smallest library user: items from stdin, the accepted value on stdout.
//! This is the snippet in `README.md`, compiled and driven by the tests.
//!
//!     printf 'alpha\nbeta\n' | cargo run --example minimal -- --inline
//!     cargo run --example minimal -- --spec layouts/dmenu.ron

use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;

use tuipalette_core::source;
use tuipalette_core::spec::Format;
use tuipalette_core::{Picker, UiSpec};
use tuipalette_tui::{Exit, Options};

/// Used when no `--spec` is given. A document can come from a file too —
/// `UiSpec::load(path, None)` infers RON or JSON from the extension.
const SPEC: &str = r#"(
    window: (title: "minimal"),
    layout: (pane: "list"),
    panes: [(id: "list", kind: Menu, title: "items")],
)"#;

fn main() -> ExitCode {
    // Two flags, hand-parsed: the point of the example is the library call,
    // not a second argument parser.
    let mut spec_path: Option<PathBuf> = None;
    let mut inline = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--spec" => spec_path = args.next().map(PathBuf::from),
            "--inline" => inline = true,
            other => {
                eprintln!("minimal: unknown argument `{other}`");
                return ExitCode::from(2);
            }
        }
    }

    // The picker draws on the terminal, so stdout has to *be* the terminal.
    // The binary solves this properly by re-pointing stdin and stdout at
    // `/dev/tty` and keeping duplicates of the pipes (crates/tuipalette/src/
    // host.rs); an example stays out of that, and says what it needs.
    if !std::io::stdout().is_terminal() {
        eprintln!("minimal: stdout is not a terminal — run this from one");
        return ExitCode::from(2);
    }

    let spec = match &spec_path {
        Some(path) => UiSpec::load(path, None),
        None => UiSpec::parse(SPEC, Format::Ron),
    };
    let spec = match spec {
        Ok(spec) => spec,
        Err(error) => {
            eprintln!("minimal: {error}");
            return ExitCode::from(1);
        }
    };
    let mut picker = match Picker::new(spec) {
        Ok(picker) => picker,
        Err(error) => {
            eprintln!("minimal: {error}");
            return ExitCode::from(1);
        }
    };
    // Items may keep arriving: the source is a channel, drained between frames.
    let source = source::stdin();
    let options = Options {
        inline: inline.then_some(10),
        ..Options::default()
    };
    match tuipalette_tui::run(&mut picker, source, &options) {
        Ok(Exit::Accepted(value)) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        Ok(Exit::Cancelled) => ExitCode::from(130),
        Err(error) => {
            eprintln!("minimal: {error}");
            ExitCode::from(2)
        }
    }
}
