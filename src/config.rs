use std::env;
use std::process::Command;

use crate::errors::MinionError;

pub const DEFAULT_MODEL: &str = "gemini-3.6-flash";
const KEYCHAIN_SERVICE: &str = "GEMINI_API_KEY";

pub fn model() -> String {
    env::var("MINION_MODEL")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_MODEL.to_owned())
}

pub fn gemini_api_key() -> Result<String, MinionError> {
    if let Some(key) = env::var("GEMINI_API_KEY")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
    {
        return Ok(key);
    }

    if let Some(account) = env::var("USER")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        && let Ok(output) = Command::new("security")
            .args([
                "find-generic-password",
                "-a",
                &account,
                "-s",
                KEYCHAIN_SERVICE,
                "-w",
            ])
            .output()
        && output.status.success()
        && let Ok(stdout) = String::from_utf8(output.stdout)
    {
        let key = stdout.trim();
        if !key.is_empty() {
            return Ok(key.to_owned());
        }
    }

    Err(MinionError::message(
        "Gemini API key not found in environment or macOS Keychain.",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_model_is_current_python_default() {
        assert_eq!(DEFAULT_MODEL, "gemini-3.6-flash");
    }
}
