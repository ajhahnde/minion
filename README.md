# Minion

Small AI-powered command-line helpers written in Rust and powered by the Gemini API.

## Features

- `ask` — process text with an optional instruction
- `rewrite` — improve or rewrite text
- `summarize` — summarize text
- `translate` — translate text into another language
- Input via argument, stdin or file

## Installation

Requires a current Rust toolchain.

```bash
cargo install --path .
```

## Configuration

Set your Gemini API key:

```bash
export GEMINI_API_KEY="..."
```

On macOS, Minion can also read the key from the Keychain:

```bash
security add-generic-password \
  -a "$USER" \
  -s GEMINI_API_KEY \
  -w "your-api-key"
```

Optionally override the default model:

```bash
export MINION_MODEL="gemini-3.6-flash"
```

## Usage

```bash
minion ask "Explain Rust ownership briefly"

minion rewrite "this text could be better" --tone professional --length shorter

minion summarize --file notes.txt --output-format bullet

echo "Long text..." | minion summarize

minion translate "Hello world" --to German

minion translate "bank" --to German --context
```

For all available options:

```bash
minion --help
minion <command> --help
```

## Development

```bash
cargo test
cargo run -- --help
```
