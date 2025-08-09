# Rust Port of llm-client

This is a Rust port (initial scaffold) of the JavaScript CLI in `javascript/`.
It mirrors the core structure: interactive API selection, chat session handling,
and message logging. API calls are currently stubbed (no outbound requests).

## Features
- Interactive selection of API provider (OpenAI, Google, Anthropic, Perplexity, Mistral, Ollama)
- Chat or multi-line input (opens your `$EDITOR` when available)
- File/Directory/Web/Image/XLSX/Git/PDF commands
- Optional web image scraping and PDF image extraction (JPEG + FlateDecode w/ PNG predictors; CMYK supported; basic JPXDecode)
- Session file management under `~/.chatgpt-client` (or Termux path)
- Message transformation templates modeled after the JS version
- Review-and-send step: optional directive, token/cost estimates, preview, confirm

## Not Yet Ported / Known gaps
- PDF JPXDecode (JPEG2000) embedded as data URLs (no raster conversion)
- Terminal markdown rendering is plain (no ANSI formatting)
- Some edge color spaces/predictors in PDFs may be skipped

## Getting Started
1. Ensure Rust toolchain is installed.
2. In this `rust/` folder, set env vars as needed (see below) or create a `.env`.
3. Build and run:
   - `cargo run --release`

## Environment Variables
- `OPENAI_API_KEY`
- `GOOGLE_AI_API_KEY`
- `ANTHROPIC_API_KEY`
- `PERPLEXITY_API_KEY`
- `MISTRAL_API_KEY`
- `OLLAMA_API_KEY` (placeholder; not currently used)

You can set these in your shell or in a `.env` file placed in the current directory when running.

## Next Steps
- Add JPEG2000 (JPXDecode) support for PDFs
- Optional terminal markdown renderer for nicer output
- Add small smoke tests for core command flows

---
This port intentionally keeps parity in user flow and data structures,
minimizing surprises for users switching from the Node.js version.
