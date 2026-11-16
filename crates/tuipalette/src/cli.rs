//! Argument parsing, hand-rolled on purpose: item 07 adds the real flags, and
//! a derive macro would only have to be unlearned before then.
//!
//! Exit codes today: 0 the spec is fine, 1 the spec is not, 2 the invocation is
//! wrong. Item 07 adds the picker's own 0/130 pair on top of that.

use std::path::PathBuf;
use std::process::ExitCode;

use tuipalette_core::spec::{Format, UiSpec};

pub const USAGE: &str = "usage: tuipalette --check --spec <path> [--format ron|json]";

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Args {
    pub spec: Option<PathBuf>,
    pub format: Option<Format>,
    pub check: bool,
}

pub fn run(argv: Vec<String>) -> Result<ExitCode, String> {
    let args = parse(&argv)?;
    if !args.check {
        return Err("only --check exists so far; the picker arrives in items 05-08".to_owned());
    }
    let path = args.spec.ok_or("--check needs --spec <path>")?;

    match UiSpec::load(&path, args.format) {
        Ok(spec) => {
            println!(
                "{}: ok — {} panes, {} bindings",
                path.display(),
                spec.panes.len(),
                spec.keys.len()
            );
            Ok(ExitCode::SUCCESS)
        }
        Err(error) => {
            eprintln!("{}: {error}", path.display());
            Ok(ExitCode::from(1))
        }
    }
}

fn parse(argv: &[String]) -> Result<Args, String> {
    let mut args = Args::default();
    let mut rest = argv.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--check" => args.check = true,
            "--spec" => {
                args.spec = Some(PathBuf::from(
                    rest.next().ok_or("--spec needs a path after it")?,
                ));
            }
            "--format" => {
                let name = rest.next().ok_or("--format needs ron or json")?;
                args.format = Some(
                    Format::from_name(name).ok_or_else(|| format!("unknown format ``{name}`"))?,
                );
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_parse(argv: &[&str]) -> Result<Args, String> {
        parse(&argv.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn parses_the_flags_it_documents() {
        let args = run_parse(&["--check", "--spec", "a.ron", "--format", "json"]).unwrap();
        assert!(args.check);
        assert_eq!(args.spec, Some(PathBuf::from("a.ron")));
        assert_eq!(args.format, Some(Format::Json));
    }

    #[test]
    fn refuses_what_it_does_not_understand() {
        assert!(run_parse(&["--nope"]).unwrap_err().contains("unknown argument"));
        assert!(run_parse(&["--spec"]).unwrap_err().contains("needs a path"));
        assert!(run_parse(&["--format", "yaml"]).unwrap_err().contains("unknown format"));
    }
}
