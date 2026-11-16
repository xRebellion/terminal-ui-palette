//! The binary. Item 07 owns the CLI contract; today it has `--check` and nothing
//! else, which is all item 03 needs to prove the error path.

mod cli;

use std::process::ExitCode;

fn main() -> ExitCode {
    match cli::run(std::env::args().skip(1).collect()) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("tuipalette: {message}");
            eprintln!("{}", cli::USAGE);
            ExitCode::from(2)
        }
    }
}
