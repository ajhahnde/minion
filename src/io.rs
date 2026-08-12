use std::io::{self, IsTerminal, Read, Write};

use crate::errors::MinionError;

pub fn read_stdin() -> Result<Option<String>, MinionError> {
    let stdin = io::stdin();
    if stdin.is_terminal() {
        return Ok(None);
    }

    let mut text = String::new();
    stdin
        .lock()
        .read_to_string(&mut text)
        .map_err(map_io_error)?;

    if text.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

pub fn write_text(text: &str) -> Result<(), MinionError> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    out.write_all(text.as_bytes()).map_err(map_io_error)?;
    if !text.ends_with('\n') {
        out.write_all(b"\n").map_err(map_io_error)?;
    }
    out.flush().map_err(map_io_error)
}

pub fn write_markdown(text: &str) -> Result<(), MinionError> {
    // Pipes must always receive the original model output. Interactive Markdown
    // rendering is intentionally kept behind this function so it can be swapped
    // for a renderer without touching command behavior.
    write_text(text)
}

fn map_io_error(err: io::Error) -> MinionError {
    if err.kind() == io::ErrorKind::BrokenPipe {
        MinionError::BrokenPipe
    } else {
        MinionError::message(err.to_string())
    }
}
