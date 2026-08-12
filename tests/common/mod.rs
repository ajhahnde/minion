use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn run_minion(args: &[&str], stdin: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_minion"));
    command
        .args(args)
        .env_remove("GEMINI_API_KEY")
        .env_remove("MINION_MODEL")
        .env_remove("USER")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    match stdin {
        Some(input) => {
            let mut child = command
                .stdin(Stdio::piped())
                .spawn()
                .expect("minion binary should start");

            child
                .stdin
                .take()
                .expect("stdin pipe should exist")
                .write_all(input.as_bytes())
                .expect("test input should be writable");

            child
                .wait_with_output()
                .expect("minion process should finish")
        }
        None => command
            .stdin(Stdio::null())
            .output()
            .expect("minion binary should run"),
    }
}

pub fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

pub struct TempFile {
    path: PathBuf,
}

impl TempFile {
    pub fn new(contents: &str) -> Self {
        let path = unique_temp_path("txt");
        fs::write(&path, contents).expect("temporary file should be writable");
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn create() -> Self {
        let path = unique_temp_path("dir");
        fs::create_dir(&path).expect("temporary directory should be creatable");
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub fn unique_temp_path(suffix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();

    std::env::temp_dir().join(format!(
        "minion-integration-test-{}-{nanos}.{suffix}",
        std::process::id()
    ))
}
