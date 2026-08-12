use std::fs;
use std::path::{Path, PathBuf};

use crate::errors::MinionError;

pub fn resolve_text(
    argument: Option<&str>,
    stdin: Option<&str>,
    file: Option<&Path>,
) -> Result<String, MinionError> {
    let has_argument = argument.is_some_and(|value| !value.trim().is_empty());
    let has_stdin = stdin.is_some_and(|value| !value.trim().is_empty());
    let has_file = file.is_some();

    let sources = [has_argument, has_stdin, has_file]
        .into_iter()
        .filter(|present| *present)
        .count();

    if sources > 1 {
        return Err(MinionError::message(
            "Provide text as an argument, via stdin, or with --file, not multiple.",
        ));
    }

    if has_argument {
        return Ok(argument.expect("checked above").to_owned());
    }

    if has_stdin {
        return Ok(stdin.expect("checked above").to_owned());
    }

    if let Some(path) = file {
        return read_file(path);
    }

    Err(MinionError::message(
        "Provide text as an argument, pipe it via stdin, or use --file.",
    ))
}

pub fn validate_file_path(value: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(value);
    let metadata =
        fs::metadata(&path).map_err(|_| format!("File not found: {}", path.display()))?;

    if !metadata.is_file() {
        return Err(format!("Not a file: {}", path.display()));
    }

    fs::File::open(&path).map_err(|err| format!("Cannot read file {}: {err}", path.display()))?;

    Ok(path)
}

fn read_file(path: &Path) -> Result<String, MinionError> {
    let bytes = fs::read(path).map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            MinionError::message(format!("File not found: {}", path.display()))
        } else {
            MinionError::message(format!("Cannot read file {}: {err}", path.display()))
        }
    })?;

    let content = String::from_utf8(bytes).map_err(|err| {
        MinionError::message(format!("Cannot read file {}: {err}", path.display()))
    })?;

    if content.trim().is_empty() {
        return Err(MinionError::message(format!(
            "File is empty: {}",
            path.display()
        )));
    }

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file(contents: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("minion-test-{suffix}.txt"));
        fs::write(&path, contents).expect("write test file");
        path
    }

    #[test]
    fn preserves_argument() {
        assert_eq!(
            resolve_text(Some("  Hello  "), None, None).unwrap(),
            "  Hello  "
        );
    }

    #[test]
    fn preserves_stdin() {
        assert_eq!(
            resolve_text(None, Some("  Hello\n"), None).unwrap(),
            "  Hello\n"
        );
    }

    #[test]
    fn ignores_blank_source() {
        assert_eq!(
            resolve_text(Some("  "), Some("stdin"), None).unwrap(),
            "stdin"
        );
    }

    #[test]
    fn rejects_multiple_sources() {
        let err = resolve_text(Some("argument"), Some("stdin"), None).unwrap_err();
        assert!(err.to_string().contains("not multiple"));
    }

    #[test]
    fn requires_input() {
        let err = resolve_text(None, None, None).unwrap_err();
        assert!(err.to_string().contains("Provide text"));
    }

    #[test]
    fn reads_file_verbatim() {
        let path = temp_file("File content\n");
        assert_eq!(
            resolve_text(None, None, Some(&path)).unwrap(),
            "File content\n"
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn rejects_blank_file() {
        let path = temp_file("   \n  \n");
        let err = resolve_text(None, None, Some(&path)).unwrap_err();
        assert!(err.to_string().contains("File is empty"));
        let _ = fs::remove_file(path);
    }
}
