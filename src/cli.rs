use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::ai;
use crate::errors::MinionError;
use crate::inputs::{resolve_text, validate_file_path};
use crate::io::{read_stdin, write_markdown, write_text};
use crate::prompts::{
    RewriteLength, RewriteTone, SummarizeOutputFormat, build_ask_prompt, build_rewrite_prompt,
    build_summarize_prompt, build_translate_prompt,
};

#[derive(Debug, Parser)]
#[command(
    name = "minion",
    version,
    about = "Small AI-powered command line helpers.",
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Show the installed Minion version.
    Version,
    /// Ask the AI to process text or piped input.
    Ask(TextOptions),
    /// Conservatively improve text or apply a rewrite instruction.
    Rewrite(RewriteOptions),
    /// Summarize text or piped input.
    Summarize(SummarizeOptions),
    /// Translate text or piped input into another language.
    Translate(TranslateOptions),
}

#[derive(Debug, Args)]
struct TextOptions {
    /// Input text. Alternatively, pipe text via stdin.
    text: Option<String>,

    /// Additional instruction for processing the input.
    #[arg(short = 'i', long)]
    instruction: Option<String>,

    /// Read input from a text file instead of an argument or stdin.
    #[arg(short = 'f', long, value_name = "FILE", value_parser = validate_file_path)]
    file: Option<PathBuf>,

    /// Render Markdown when writing to an interactive terminal.
    #[arg(long)]
    markdown: bool,
}

#[derive(Debug, Args)]
struct RewriteOptions {
    /// Input text. Alternatively, pipe text via stdin.
    text: Option<String>,

    /// Additional instruction for processing the input.
    #[arg(short = 'i', long)]
    instruction: Option<String>,

    /// Target tone. Omit to preserve it unless --instruction changes it.
    #[arg(long, value_enum, ignore_case = true)]
    tone: Option<RewriteTone>,

    /// Target length. Omit to preserve it unless --instruction changes it.
    #[arg(long, value_enum, ignore_case = true)]
    length: Option<RewriteLength>,

    /// Read input from a text file instead of an argument or stdin.
    #[arg(short = 'f', long, value_name = "FILE", value_parser = validate_file_path)]
    file: Option<PathBuf>,

    /// Render Markdown when writing to an interactive terminal.
    #[arg(long)]
    markdown: bool,
}

#[derive(Debug, Args)]
struct SummarizeOptions {
    /// Input text. Alternatively, pipe text via stdin.
    text: Option<String>,

    /// Additional instruction for processing the input.
    #[arg(short = 'i', long)]
    instruction: Option<String>,

    /// Output format. Omit to use the default.
    #[arg(short = 'o', long = "output-format", value_enum, ignore_case = true)]
    output_format: Option<SummarizeOutputFormat>,

    /// Read input from a text file instead of an argument or stdin.
    #[arg(short = 'f', long, value_name = "FILE", value_parser = validate_file_path)]
    file: Option<PathBuf>,

    /// Render Markdown when writing to an interactive terminal.
    #[arg(long)]
    markdown: bool,
}

#[derive(Debug, Args)]
struct TranslateOptions {
    /// Input text. Alternatively, pipe text via stdin.
    text: Option<String>,

    /// Additional instruction for processing the input.
    #[arg(short = 'i', long)]
    instruction: Option<String>,

    /// Language to translate into.
    #[arg(short = 't', long = "to", required = true)]
    target_language: String,

    /// Show common translations for different usage contexts.
    #[arg(long)]
    context: bool,

    /// Read input from a text file instead of an argument or stdin.
    #[arg(short = 'f', long, value_name = "FILE", value_parser = validate_file_path)]
    file: Option<PathBuf>,

    /// Render Markdown when writing to an interactive terminal.
    #[arg(long)]
    markdown: bool,
}

pub fn run() -> Result<(), MinionError> {
    run_with(Cli::parse(), ai::ask)
}

fn run_with<F>(cli: Cli, ask_ai: F) -> Result<(), MinionError>
where
    F: Fn(&str) -> Result<String, MinionError>,
{
    match cli.command {
        Command::Version => write_text(env!("CARGO_PKG_VERSION")),
        Command::Ask(options) => {
            let input = resolve_command_input(options.text.as_deref(), options.file.as_deref())?;
            let prompt = build_ask_prompt(options.instruction.as_deref(), Some(&input))?;
            write_response(&ask_ai(&prompt)?, options.markdown)
        }
        Command::Rewrite(options) => {
            let input = resolve_command_input(options.text.as_deref(), options.file.as_deref())?;
            let prompt = build_rewrite_prompt(
                &input,
                options.instruction.as_deref(),
                options.tone,
                options.length,
            )?;
            write_response(&ask_ai(&prompt)?, options.markdown)
        }
        Command::Summarize(options) => {
            let input = resolve_command_input(options.text.as_deref(), options.file.as_deref())?;
            let prompt = build_summarize_prompt(
                &input,
                options.instruction.as_deref(),
                options.output_format,
            )?;
            write_response(&ask_ai(&prompt)?, options.markdown)
        }
        Command::Translate(options) => {
            let input = resolve_command_input(options.text.as_deref(), options.file.as_deref())?;
            let prompt = build_translate_prompt(
                &input,
                &options.target_language,
                options.context,
                options.instruction.as_deref(),
            )?;
            write_response(&ask_ai(&prompt)?, options.markdown)
        }
    }
}

fn resolve_command_input(
    text: Option<&str>,
    file: Option<&std::path::Path>,
) -> Result<String, MinionError> {
    let stdin = read_stdin()?;
    resolve_text(text, stdin.as_deref(), file)
}

fn write_response(response: &str, markdown: bool) -> Result<(), MinionError> {
    if markdown {
        write_markdown(response)
    } else {
        write_text(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn clap_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn uppercase_tone_is_accepted() {
        let cli =
            Cli::try_parse_from(["minion", "rewrite", "Some text", "--tone", "FRIENDLY"]).unwrap();

        match cli.command {
            Command::Rewrite(options) => assert_eq!(options.tone, Some(RewriteTone::Friendly)),
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn translate_requires_target_language() {
        let err = Cli::try_parse_from(["minion", "translate", "Hello"]).unwrap_err();
        assert_eq!(err.exit_code(), 2);
    }
}
