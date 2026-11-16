//! `tuipalette` — the reference binary: a document in, a line out, and the same
//! exit codes dmenu callers already handle (docs/PLAN.md, 07).

mod cli;
mod host;

use std::io::{self, BufReader, Write};
use std::process::ExitCode;
use std::sync::mpsc;
use std::time::Instant;

use tuipalette_core::spec::{Kind, Theme};
use tuipalette_core::{source, Picker, UiSpec};

use cli::{Args, Stream};

fn main() -> ExitCode {
    let started = Instant::now();
    let timing = std::env::var_os("TUIPALETTE_TIMING").is_some();
    let args = match cli::parse(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("tuipalette: {message}");
            eprint!("{}", cli::USAGE);
            return ExitCode::from(2);
        }
    };
    match run(args, started, timing) {
        Ok(code) => ExitCode::from(code),
        Err((code, message)) => {
            eprintln!("tuipalette: {message}");
            ExitCode::from(code)
        }
    }
}

fn run(args: Args, started: Instant, timing: bool) -> Result<u8, (u8, String)> {
    if args.help {
        print!("{}", cli::USAGE);
        return Ok(0);
    }
    if args.version {
        println!("tuipalette {}", tuipalette_core::VERSION);
        return Ok(0);
    }
    let path = args
        .spec
        .clone()
        .ok_or((2, "--spec <file> is required".to_owned()))?;
    let mut spec = UiSpec::load(&path, args.format).map_err(|error| (1, error.to_string()))?;
    if timing {
        eprintln!("phase spec-parse {:.1}ms", millis(started));
    }

    if let Some(path) = &args.theme {
        spec.theme = Theme::load(path).map_err(|error| (1, error.to_string()))?;
    }
    if let Some(prompt) = &args.prompt {
        for pane in spec.panes.iter_mut().filter(|pane| pane.kind == Kind::Input) {
            pane.title.clone_from(prompt);
        }
    }
    if let Some(message) = &args.message {
        // rofi's -mesg: one hint line, shown where the hints would be.
        for pane in spec.panes.iter_mut().filter(|pane| pane.kind == Kind::Status) {
            pane.items = vec![message.clone()];
        }
    }
    if let Some(icons) = args.icons {
        for pane in spec.panes.iter_mut().filter(|pane| pane.kind == Kind::Menu) {
            pane.icon_width = icons.then(|| pane.icon_width.unwrap_or(2));
        }
    }

    if args.check {
        println!(
            "{}: ok — {} panes, {} bindings",
            path.display(),
            spec.panes.len(),
            spec.keys.len()
        );
        return Ok(0);
    }
    if args.print_class {
        println!(
            "{}",
            spec.host.class.clone().unwrap_or_else(|| "tuipalette".to_owned())
        );
        return Ok(0);
    }
    if args.print_geometry {
        let geometry = spec.geometry().ok_or((
            1,
            "the document declares no host.width and host.height".to_owned(),
        ))?;
        println!("{geometry}");
        return Ok(0);
    }

    let host = host::Host::prepare().map_err(|error| (2, error.to_string()))?;
    let mut picker = Picker::new(spec)
        .map_err(|error| (1, error.to_string()))?
        .no_sort(args.no_sort)
        .accept_query(args.accept_query);
    let (_keepalive, disconnected) = mpsc::channel();
    let source = match host.items {
        Some(file) => match args.input {
            Stream::Line => source::stream(BufReader::new(file)),
            Stream::Json => source::stream_json(BufReader::new(file)),
        },
        None => disconnected,
    };

    let height = args
        .height
        .or(picker.session().spec().host.height)
        .or_else(|| tuipalette_tui::terminal_rows())
        .unwrap_or(10);
    // Inline is opt-in. It used to be the default whenever stdout was a tty, which
    // made --inline a no-op and put a dedicated floating window into inline mode.
    let inline = if args.inline { Some(height) } else { None };
    let options = tuipalette_tui::Options {
        inline,
        timing: timing.then_some(started),
    };
    let outcome = tuipalette_tui::run(&mut picker, source, &options)
        .map_err(|error| (2, error.to_string()))?;

    let mut out: Box<dyn Write> = match (args.out_file.as_ref(), host.value_out) {
        (Some(path), _) => Box::new(
            std::fs::File::create(path).map_err(|error| (2, format!("{}: {error}", path.display())))?,
        ),
        (None, Some(file)) => Box::new(file),
        (None, None) => Box::new(io::stdout()),
    };
    match outcome {
        tuipalette_tui::Exit::Accepted(value) => {
            write_result(&mut out, &args, picker.query(), &value)
                .map_err(|error| (2, error.to_string()))?;
            Ok(0)
        }
        tuipalette_tui::Exit::Cancelled => Ok(130),
    }
}

/// The value, in the shape the caller asked for. A line is dmenu's answer; the
/// JSON form is how a value with a newline in it survives (D16).
fn write_result(
    out: &mut dyn Write,
    args: &Args,
    query: &str,
    value: &str,
) -> io::Result<()> {
    match args.output {
        Stream::Line => {
            if args.print_query {
                writeln!(out, "{query}")?;
            }
            writeln!(out, "{value}")?;
        }
        Stream::Json => {
            let result = serde_json::json!({ "query": query, "value": value });
            writeln!(out, "{result}")?;
        }
    }
    out.flush()
}

fn millis(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}
