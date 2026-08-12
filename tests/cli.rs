mod common;

use std::fs;

use common::{TempDir, TempFile, run_minion, stderr, stdout, unique_temp_path};

#[test]
fn version_subcommand_prints_package_version() {
    let output = run_minion(&["version"], None);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), format!("{}\n", env!("CARGO_PKG_VERSION")));
    assert_eq!(stderr(&output), "");
}

#[test]
fn version_flag_prints_package_version() {
    let output = run_minion(&["--version"], None);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains(env!("CARGO_PKG_VERSION")));
    assert_eq!(stderr(&output), "");
}

#[test]
fn help_lists_the_public_commands() {
    let output = run_minion(&["--help"], None);
    let out = stdout(&output);

    assert_eq!(output.status.code(), Some(0));
    assert!(out.contains("Usage:"));
    assert!(out.contains("ask"));
    assert!(out.contains("rewrite"));
    assert!(out.contains("summarize"));
    assert!(out.contains("translate"));
    assert!(out.contains("version"));
    assert_eq!(stderr(&output), "");
}

#[test]
fn missing_subcommand_fails_with_usage_information() {
    let output = run_minion(&[], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(2));
    assert!(err.contains("Usage:"));
}

#[test]
fn unknown_subcommand_is_rejected_by_clap() {
    let output = run_minion(&["definitely-not-a-command"], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(2));
    assert!(err.contains("unrecognized subcommand"));
}

#[test]
fn translate_requires_target_language() {
    let output = run_minion(&["translate", "Hello"], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(2));
    assert!(err.contains("--to"));
}

#[test]
fn unknown_rewrite_tone_is_rejected() {
    let output = run_minion(&["rewrite", "Hello", "--tone", "dramatic"], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(2));
    assert!(err.contains("invalid value"));
    assert!(err.contains("friendly"));
}

#[test]
fn rewrite_tone_is_case_insensitive() {
    let output = run_minion(&["rewrite", "Hello", "--tone", "FRIENDLY"], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("Gemini API key not found"));
}

#[test]
fn missing_input_is_reported_before_contacting_the_api() {
    let output = run_minion(&["ask"], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("Provide text as an argument"));
}

#[test]
fn blank_stdin_counts_as_missing_input() {
    let output = run_minion(&["ask"], Some("  \n\t\n"));
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("Provide text as an argument"));
}

#[test]
fn argument_and_stdin_are_mutually_exclusive() {
    let output = run_minion(&["ask", "argument text"], Some("stdin text\n"));
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("not multiple"));
}

#[test]
fn argument_and_file_are_mutually_exclusive() {
    let file = TempFile::new("file text\n");
    let path = file.path().to_string_lossy().into_owned();
    let output = run_minion(&["ask", "argument text", "--file", &path], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("not multiple"));
}

#[test]
fn nonexistent_file_is_rejected_during_argument_parsing() {
    let path = unique_temp_path("missing.txt");
    let path = path.to_string_lossy().into_owned();
    let output = run_minion(&["ask", "--file", &path], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(2));
    assert!(err.contains("File not found:"));
}

#[test]
fn directory_is_rejected_where_a_file_is_required() {
    let directory = TempDir::create();
    let path = directory.path().to_string_lossy().into_owned();
    let output = run_minion(&["ask", "--file", &path], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(2));
    assert!(err.contains("Not a file:"));
}

#[test]
fn empty_file_is_rejected_as_input() {
    let file = TempFile::new(" \n\t\n");
    let path = file.path().to_string_lossy().into_owned();
    let output = run_minion(&["ask", "--file", &path], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("File is empty:"));
}

#[test]
fn valid_file_input_reaches_api_configuration() {
    let file = TempFile::new("hello from a file\n");
    let path = file.path().to_string_lossy().into_owned();
    let output = run_minion(&["ask", "--file", &path], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("Gemini API key not found"));
}

#[test]
fn blank_ask_instruction_is_rejected_before_api_configuration() {
    let output = run_minion(&["ask", "Hello", "--instruction", "   "], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("Instruction must not be empty"));
}

#[test]
fn blank_rewrite_instruction_is_rejected_before_api_configuration() {
    let output = run_minion(&["rewrite", "Hello", "--instruction", "   "], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("Rewrite instruction must not be empty"));
}

#[test]
fn blank_summarize_instruction_is_rejected_before_api_configuration() {
    let output = run_minion(&["summarize", "Hello", "--instruction", "   "], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("Summarize instruction must not be empty"));
}

#[test]
fn blank_translate_instruction_is_rejected_before_api_configuration() {
    let output = run_minion(
        &[
            "translate",
            "Hello",
            "--to",
            "German",
            "--instruction",
            "   ",
        ],
        None,
    );
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("Translate instruction must not be empty"));
}

#[test]
fn blank_target_language_is_rejected_before_api_configuration() {
    let output = run_minion(&["translate", "Hello", "--to", "   "], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("Target language must not be empty"));
}

#[test]
fn stdin_input_reaches_api_configuration() {
    let output = run_minion(&["summarize"], Some("Text from stdin\n"));
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("Gemini API key not found"));
}

#[test]
fn unreadable_utf8_file_is_reported_as_an_input_error() {
    let path = unique_temp_path("invalid-utf8.txt");
    fs::write(&path, [0xff, 0xfe, 0xfd]).expect("invalid UTF-8 fixture should be writable");
    let path_string = path.to_string_lossy().into_owned();

    let output = run_minion(&["ask", "--file", &path_string], None);
    let err = stderr(&output);

    let _ = fs::remove_file(path);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("Cannot read file"));
}

#[test]
fn valid_ask_text_reaches_api_configuration() {
    let output = run_minion(&["ask", "Hello"], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("Gemini API key not found"));
}

#[test]
fn rewrite_length_is_case_insensitive() {
    let output = run_minion(&["rewrite", "Hello", "--length", "SHORTER"], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("Gemini API key not found"));
}

#[test]
fn unknown_rewrite_length_is_rejected() {
    let output = run_minion(&["rewrite", "Hello", "--length", "tiny"], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(2));
    assert!(err.contains("invalid value"));
    assert!(err.contains("shorter"));
}

#[test]
fn summarize_output_format_is_case_insensitive() {
    let output = run_minion(&["summarize", "Hello", "--output-format", "BULLET"], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("Gemini API key not found"));
}

#[test]
fn unknown_summarize_output_format_is_rejected() {
    let output = run_minion(&["summarize", "Hello", "--output-format", "table"], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(2));
    assert!(err.contains("invalid value"));
    assert!(err.contains("bullet"));
}

#[test]
fn translate_context_flag_is_accepted() {
    let output = run_minion(&["translate", "bank", "--to", "German", "--context"], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("Gemini API key not found"));
}

#[test]
fn short_target_language_flag_is_accepted() {
    let output = run_minion(&["translate", "Hello", "-t", "German"], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("Gemini API key not found"));
}

#[test]
fn file_and_stdin_are_mutually_exclusive() {
    let file = TempFile::new("file text\n");
    let path = file.path().to_string_lossy().into_owned();
    let output = run_minion(&["ask", "--file", &path], Some("stdin text\n"));
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("not multiple"));
}

#[test]
fn each_processing_subcommand_has_help() {
    for command in ["ask", "rewrite", "summarize", "translate"] {
        let output = run_minion(&[command, "--help"], None);
        let out = stdout(&output);

        assert_eq!(output.status.code(), Some(0), "help failed for {command}");
        assert!(out.contains("Usage:"), "missing usage for {command}");
        assert_eq!(stderr(&output), "", "unexpected stderr for {command}");
    }
}

#[test]
fn markdown_flag_is_accepted() {
    let output = run_minion(&["ask", "Hello", "--markdown"], None);
    let err = stderr(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(err.contains("Gemini API key not found"));
}
