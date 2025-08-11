# Rust llm-client

---

## **Project Overview**

This is a **Rust port of an AI chat client CLI**, originally in JavaScript. It interacts with multiple LLM (large language model) APIs (OpenAI, Google Gemini, Anthropic Claude, Perplexity, Mistral, and Ollama) and supports uploading, analyzing, and extracting text from various sources (files, directories, web pages, images, PDFs, xlsx, git repos, etc).

The user interacts through an interactive prompt, picks a model and a command, reviews a preview, and sends that information to the LLM API, viewing the response in the terminal with pretty markdown rendering and cost/token estimates.

## **Key Features**

- **Choose Language Model/API provider** (OpenAI, Google Gemini, Claude, etc)
- **Send chat or multi-line messages**
- **Attach/extract information from files, directories, URLs, images, PDFs, Excel workbooks, Git repos**
- **Handles image and PDF extraction including embedded images, with token/cost estimation**
- **Maintains a session log (`message-log-...md`) per session**
- **Optionally saves or discards the chat log on exit**
- **Interactive review step before sending to the LLM**
- **Pretty terminal markdown rendering (via `termimad`)**

## Known gaps
- PDF JPXDecode (JPEG2000) embedded as data URLs (no raster conversion)
- Terminal markdown rendering is plain (no ANSI formatting)
- Some edge color spaces/predictors in PDFs may be skipped

## **How It Works**

1. **Startup & Environment**
   - `.env` or environment variables provide API keys.
   - Optional request timeout via `REQUEST_TIMEOUT_SECS` (default 60).
   - Session directories are created under `~/.chatgpt-client/` (or Termux location).

2. **Interactive Loop (`src/ui.rs`)**
   - Presents a menu: choose LLM provider (`API_CHOICES`), command type (chat, file, web, etc).
   - Processes user input appropriately (chat, read from file, scrape web, etc).
   - Extracts messages using `extractor` modules (web, pdf, xlsx, etc).
   - Runs a "review-and-send" step: token/cost preview, confirm to send, show data.
   - Extends the log with message(s), sends to the provider API, displays output, saves to session log.
   - On exit/save: cleans up temp files and (optionally) logs the transcript.

3. **API Wrappers (`api/mod.rs`)**
   - For each provider (OpenAI, Anthropic, Google, etc), a struct implements `ApiClient` trait for sending messages.
   - Converts chat log into provider-specific JSON using templates.
   - Handles result parsing, errors, and returns raw/pretty response.

4. **Extractors**
   - **File/Dir** (`extractor/file.rs`, `extractor/dir.rs`): reads plain text, walks directories (with exclusion filters).
   - **Web** (`extractor/web.rs`): fetches HTML, extracts text and images recursively, host-limited.
   - **PDF** (`extractor/pdf.rs`): uses `pdf-extract` for text, plus `lopdf`/`image` to pull embedded images and convert to JPEG, base64-encoded as data URLs.
   - **Image** (`extractor/image.rs`), **XLSX** (`extractor/xlsx.rs`)
   - **Git repos**: clones to temp folder, then walks the directory.

5. **Session (`session.rs`)**
   - Manages session-specific storage (files/temp/chats dirs), handles message log appending, copies files, downloads from URLs, cleans up.

6. **Message Log (`message_log.rs`)**
   - Tracks all messages in the current chat: user/model roles, text/image types.

7. **Markdown rendering (`markdown.rs`)**
   - Prints markdown in plain or colored/pretty terminal format using Termimad.

## **Important Files, Summarized**

- **Cargo.toml**: Rust dependencies — includes HTTP, JSON, PDF/image processing, UI prompts, etc.
- **README.md**: Project description, usage, setup, and current limitations.
- **.env / .env.example**: API keys for providers; `REQUEST_TIMEOUT_SECS` to set default HTTP timeout.
- **src/main.rs**: Entry point; boots up the CLI app.
- **src/ui.rs**: Main interactive loop for all commands and session handling.
- **src/api/**: Provider-specific API client code and JSON message transformation logic.
- **src/extractor/**: Modules to extract/convert content from files, web, images, PDFs, Excel, git, etc.
- **src/session.rs**: Session management (file structures, temp download, log file write).
- **src/message_log.rs**: Tracks all user + model messages, roles, and types.
- **src/markdown.rs**: Pretty terminal markdown print.

## **A Typical Session**

1. You pick your LLM provider.
2. You choose a command (chat, file, web, directory, git, etc).
3. If it needs a file/URL/etc, you supply it. The extractor module parses or downloads as needed.
4. You review a summary: tokens/cost, extracted text preview (if you want).
5. You confirm to send (or cancel).
6. The data is sent to the API, and a markdown-formatted reply is shown.
7. This loop continues; on exit, you can keep or delete the session log.
