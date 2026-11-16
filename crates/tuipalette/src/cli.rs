//! The flags. Hand-rolled on purpose: a picker on a keybind pays for every
//! argument library in start-up time, and the whole surface is a dozen flags
//! (docs/PLAN.md, 07).

use std::path::PathBuf;

use tuipalette_core::spec::Format;

/// Which way items are written on stdin, and the value on stdout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    /// `display[\tvalue[\ticon]]` — the dmenu-compatible default.
    Line,
    /// One JSON object per line: `{"display":..,"value":..,"icon":..}`.
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub spec: Option<PathBuf>,
    pub format: Option<Format>,
    pub check: bool,
    pub inline: bool,
    pub height: Option<u16>,
    pub print_query: bool,
    pub print_geometry: bool,
    pub print_class: bool,
    /// Write the value here instead of stdout: a floating host has no stdout to
    /// give the picker, so the adapter names a file and reads it after.
    pub out_file: Option<PathBuf>,
    pub input: Stream,
    pub output: Stream,
    pub prompt: Option<String>,
    pub theme: Option<PathBuf>,
    /// `Some(true)` reserves an icon column, `Some(false)` removes one.
    pub icons: Option<bool>,
    pub help: bool,
    pub version: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            spec: None,
            format: None,
            check: false,
            inline: false,
            height: None,
            print_query: false,
            print_geometry: false,
            print_class: false,
            out_file: None,
            input: Stream::Line,
            output: Stream::Line,
            prompt: None,
            theme: None,
            icons: None,
            help: false,
            version: false,
        }
    }
}

/// Short enough to read in one screen, and it names every flag.
pub const USAGE: &str = "\
tuipalette — a picker built from a RON document

usage: tuipalette --spec <file> [options]
       printf 'a\\nb\\n' | tuipalette --spec spec.ron

  --spec <file>      the document to run (required)
  --format ron|json  how to read it; inferred from the extension by default
  --check            validate the document and print what it contains
  --inline           draw inside the invoking terminal instead of the alternate screen
  --height <n>       with --inline, take n lines at the bottom
  --prompt <text>    replace the query pane's title
  --theme <file>     read the theme tokens from this RON file instead
  --icons            reserve a two-cell icon column in menu panes
  --no-icons         drop any icon column the document declares
  --in line|json     how items arrive on stdin (default line)
  --out line|json    how the chosen value leaves on stdout (default line)
  --print-query      print the query before the value
  --print-geometry   print the document's <width>x<height> and exit
  --print-class      print the window class the floating adapter should ask for
  --out-file <path>  write the chosen value here instead of stdout
  -h, --help         this
  -V, --version      version

exit: 0 accepted · 130 cancelled · 1 the document is wrong · 2 unusable host or flags
";

pub fn parse<I: IntoIterator<Item = String>>(args: I) -> Result<Args, String> {
    let mut out = Args::default();
    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        let (name, attached) = match flag.split_once('=') {
            Some((name, value)) if name.starts_with("--") => (name.to_owned(), Some(value.to_owned())),
            _ => (flag, None),
        };
        let mut take = |name: &str| -> Result<String, String> {
            match attached.clone() {
                Some(value) => Ok(value),
                None => args.next().ok_or_else(|| format!("{name} needs a value")),
            }
        };
        match name.as_str() {
            "-h" | "--help" => out.help = true,
            "-V" | "--version" => out.version = true,
            "--check" => out.check = true,
            "--inline" => out.inline = true,
            "--print-query" => out.print_query = true,
            "--print-geometry" => out.print_geometry = true,
            "--print-class" => out.print_class = true,
            "--out-file" => out.out_file = Some(PathBuf::from(take("--out-file")?)),
            "--icons" => out.icons = Some(true),
            "--no-icons" => out.icons = Some(false),
            "--spec" => out.spec = Some(PathBuf::from(take("--spec")?)),
            "--theme" => out.theme = Some(PathBuf::from(take("--theme")?)),
            "--prompt" => out.prompt = Some(take("--prompt")?),
            "--format" => {
                let text = take("--format")?;
                out.format = Some(Format::from_name(&text)
                    .ok_or_else(|| format!("{text} is not a format — expected ron or json"))?);
            }
            "--height" => {
                let text = take("--height")?;
                out.height = Some(
                    text.parse()
                        .map_err(|_| format!("{text} is not a height in lines"))?,
                );
            }
            "--in" => out.input = stream(&take("--in")?, "--in")?,
            "--out" => out.output = stream(&take("--out")?, "--out")?,
            other if other.starts_with('-') => return Err(format!("unknown flag {other}")),
            other => return Err(format!("unexpected argument {other} — items come on stdin")),
        }
    }
    Ok(out)
}

fn stream(text: &str, flag: &str) -> Result<Stream, String> {
    match text {
        "line" => Ok(Stream::Line),
        "json" => Ok(Stream::Json),
        other => Err(format!("{flag} takes line or json, not {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Result<Args, String> {
        parse(list.iter().map(|item| item.to_string()))
    }

    #[test]
    fn flags_parse_in_both_spellings() {
        let split = args(&["--spec", "a.ron", "--height", "7"]).expect("flags");
        assert_eq!(split.spec, Some(PathBuf::from("a.ron")));
        assert_eq!(split.height, Some(7));
        let joined = args(&["--spec=a.ron", "--height=7"]).expect("flags");
        assert_eq!(joined, split);
    }

    #[test]
    fn the_stream_flags_take_two_values() {
        let parsed = args(&["--in", "json", "--out", "json"]).expect("flags");
        assert_eq!(parsed.input, Stream::Json);
        assert_eq!(parsed.output, Stream::Json);
        assert!(args(&["--in", "yaml"]).is_err());
    }

    #[test]
    fn a_missing_value_is_named() {
        let error = args(&["--spec"]).expect_err("no value");
        assert!(error.contains("--spec"), "{error}");
    }

    #[test]
    fn an_unknown_flag_or_a_stray_word_is_refused() {
        assert!(args(&["--bogus"]).expect_err("unknown").contains("--bogus"));
        assert!(args(&["items.txt"]).expect_err("positional").contains("stdin"));
    }

    #[test]
    fn icons_can_be_forced_either_way() {
        assert_eq!(args(&["--icons"]).expect("flags").icons, Some(true));
        assert_eq!(args(&["--no-icons"]).expect("flags").icons, Some(false));
    }

    #[test]
    fn the_default_is_a_dmenu() {
        let parsed = args(&[]).expect("flags");
        assert_eq!(parsed.input, Stream::Line);
        assert_eq!(parsed.output, Stream::Line);
        assert!(!parsed.inline);
        assert!(!parsed.check);
    }
}
