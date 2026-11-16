//! Being run from a pipe: items on stdin, keys from the terminal, the value on
//! stdout — all three at once, which is what a dmenu-compatible picker needs and
//! what a terminal will not give you twice.
//!
//! The trick is the old one: keep duplicates of the real stdin and stdout, then
//! point both at `/dev/tty` so the terminal backend and crossterm talk to the
//! terminal while the original pipes stay open for items and for the result.

use std::fs::{File, OpenOptions};
use std::io::{self, IsTerminal};
use std::os::fd::{AsRawFd, FromRawFd};

pub struct Host {
    /// The pipe the items arrive on, when stdin was not a terminal.
    pub items: Option<File>,
    /// Where the accepted value goes, when stdout was not a terminal.
    pub value_out: Option<File>,
}

impl Host {
    /// No pipes anywhere: the terminal is already all three.
    pub fn plain() -> Self {
        Self {
            items: None,
            value_out: None,
        }
    }

    /// Take over the fds the picker needs. Fails when there is no terminal to
    /// draw on, which is the "unusable host" case (exit 2).
    pub fn prepare() -> io::Result<Self> {
        let stdin_tty = io::stdin().is_terminal();
        let stdout_tty = io::stdout().is_terminal();
        if stdin_tty && stdout_tty {
            return Ok(Self::plain());
        }
        let tty = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!("/dev/tty is not available ({error}); a picker needs a terminal"),
                )
            })?;
        let items = if stdin_tty { None } else { Some(duplicate(0)?) };
        let value_out = if stdout_tty {
            None
        } else {
            let copy = duplicate(1)?;
            redirect(1, &tty)?;
            Some(copy)
        };
        if !stdin_tty {
            redirect(0, &tty)?;
        }
        Ok(Self { items, value_out })
    }
}

fn duplicate(fd: i32) -> io::Result<File> {
    let new = unsafe { libc::dup(fd) };
    if new < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { File::from_raw_fd(new) })
}

fn redirect(target: i32, tty: &File) -> io::Result<()> {
    if unsafe { libc::dup2(tty.as_raw_fd(), target) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
