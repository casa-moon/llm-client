use anyhow::Result;
use inquire::{Confirm, Select, Text};
use image::{GenericImageView, ImageReader};
use owo_colors::OwoColorize;

use crate::api::{create_client, API_CHOICES, Client, ModelResponse};
use crate::message_log::{Message, MessageLog, MsgType, Role};
use crate::session::ChatSession;
use crate::extractor::file as file_extractor;

pub fn run_app() -> Result<()> {
  // choose API
  let choices: Vec<String> = API_CHOICES
    .iter()
    .map(|c| c.label.to_string())
    .collect();
  let selection = Select::new("Select API", choices).prompt()?;

  // Map back to key by label position
  let idx = API_CHOICES
    .iter()
    .position(|c| c.label == selection)
    .expect("choice index");
  let choice_key = API_CHOICES[idx].key;

  let (mut client, mut model) = create_client(choice_key)?;

  if matches!(client, Client::Ollama(_)) {
    // Query Ollama for available models
    let models = match crate::api::list_ollama_models() {
      Ok(v) if !v.is_empty() => v,
      _ => vec!["llama3:latest".to_string()],
    };
    let selection = Select::new("Select Ollama model", models.clone()).prompt()?;
    model = selection;
  }

  let mut session = ChatSession::new()?;
  let mut log = MessageLog::new();

  // main loop
  let commands = vec![
    ("Chat Input", "chat"),
    ("Multi-line Input", "multi"),
    ("File", "file"),
    ("Directory", "dir"),
    ("Web Page", "web"),
    ("Image", "image"),
    ("Video (Sora)", "sora_video"),
    ("PDF", "pdf"),
    ("Excel (xlsx)", "xlsx"),
    ("Git Repository", "git"),
    ("Save Chat Session", "save"),
    ("Exit Application", "exit"),
  ];
  loop {
    let choice = Select::new(
      "Select command",
      commands.iter().map(|(n, _)| n.to_string()).collect(),
    )
      .prompt()?;

    let command = commands.iter().find(|(n, _)| n == &choice).unwrap().1;
    match command {
      "save" => {
        // Try to summarize the chat via the active API for a filename slug
        let mut sum_log = MessageLog::new();
        sum_log.extend(log.raw().clone());
        sum_log.add_user("Summarize this entire conversation in <= 256 characters to create a filename that encapsulates the essence of the content. Plain text only. No quotes. No markdown. One sentence.");
        // Show a spinner while generating the summary + renaming
        let pb = crate::spinner::start("Renaming file...");
          let rename_result = (|| -> Result<()> {
          let resp = client.send_message(&model, &sum_log)?;
          // Ignore any model "thinking" content wrapped in <think>...</think>
          let mut s = strip_think_sections(&resp.text).trim().to_string();
          // Keep it within 256 chars on char boundaries
          if s.chars().count() > 256 { s = s.chars().take(256).collect(); }
          session.rename_with_summary(&s)?;
          Ok(())
        })();
        crate::spinner::stop(&pb);
        if let Err(e) = rename_result {
          println!("Could not summarize for filename: {}", e);
        }
        session.clean_up(true)?;
        break;
      }
      "exit" => {
        session.clean_up(false)?;
        break;
      }
      "chat" => {
        let input = Text::new("Chat input:").prompt()?;
        session.append_message_to_file("\n\n***\n\n### User:\n")?;
        session.append_message_to_file(&input)?;
        let temp_msgs = vec![Message { role: Role::User, kind: MsgType::Text, content: input }];
        review_and_send(temp_msgs, &mut client, &model, &mut log, &mut session, false)?;
      }
      "multi" => {
        let input = match dialoguer::Editor::new().require_save(false).edit("\n# Enter multi-line input below\n") {
          Ok(Some(s)) => s,
          _ => Text::new("Multi-line input:").prompt()?,
        };
        session.append_message_to_file("\n\n***\n\n### User:\n")?;
        session.append_message_to_file(&input)?;
        let temp_msgs = vec![Message { role: Role::User, kind: MsgType::Text, content: input }];
        review_and_send(temp_msgs, &mut client, &model, &mut log, &mut session, true)?;
      }
      "file" => {
        let raw = Text::new("Enter path:").prompt()?;
        let path_str = normalize_path_input(&raw);
        let path_abs = resolve_local_or_download(&session, &path_str)?;
        if !path_abs.is_file() {
          println!("\nFile not found.\n");
          continue;
        }
        session.append_message_to_file("\n\n***\n\n### User:\n")?;
        session.append_message_to_file(&path_abs.to_string_lossy())?;

        // Copy to session files dir if different
        let dest = session.copy_file_to_dir(&path_abs)?;

        // Extract text and review before sending
        match file_extractor::extract_text(&dest) {
          Ok(msgs) => {
            review_and_send(msgs, &mut client, &model, &mut log, &mut session, true)?;
          }
          Err(e) => {
            println!("\nFailed to extract file: {}\n", e);
          }
        }
      }
      "dir" => {
        let raw = Text::new("Enter path:").prompt()?;
        let path_str = normalize_path_input(&raw);
        let path = std::path::Path::new(&path_str);
        let path_abs = match std::fs::canonicalize(path) {
          Ok(p) => p,
          Err(_) => path.to_path_buf()
        };
        if !path_abs.is_dir() {
          println!("\nDirectory not found.\n");
          continue;
        }
        let recursive = Confirm::new("Recursive?").with_default(true).prompt()?;
        session.append_message_to_file("\n\n***\n\n### User:\n")?;
        session.append_message_to_file(&path_abs.to_string_lossy())?;
        match crate::extractor::dir::extract_dir(&session, &path_abs, recursive) {
          Ok(msgs) => { review_and_send(msgs, &mut client, &model, &mut log, &mut session, true)?; }
          Err(e) => println!("\nFailed to extract directory: {}\n", e),
        }
      }
      "web" => {
        let url = Text::new("Enter URL:").prompt()?;
        let depth_input = Text::new("Depth (0-2)").with_placeholder("0").prompt()?;
        let depth: usize = depth_input.trim().parse().unwrap_or(0);
        let get_images = Confirm::new("Get images?").with_default(false).prompt()?;
        session.append_message_to_file("\n\n***\n\n### User:\n")?;
        session.append_message_to_file(&url)?;
        let mut total: Vec<Message> = Vec::new();
        match crate::extractor::web::extract_text(&session, &url, depth) {
          Ok(mut msgs) => { total.append(&mut msgs); }
          Err(e) => println!("\nFailed to extract web text: {}\n", e),
        }
        if get_images {
          session.image_token_count = 0; // reset before counting
          match crate::extractor::web::extract_images(&mut session, &url, depth) {
            Ok(mut msgs) => { total.append(&mut msgs); }
            Err(e) => println!("\nFailed to extract web images: {}\n", e),
          }
        }
        review_and_send(total, &mut client, &model, &mut log, &mut session, true)?;
      }
      "image" => {
        let url = Text::new("Enter image URL:").prompt()?;
        session.append_message_to_file("\n\n***\n\n### User:\n")?;
        session.append_message_to_file(&url)?;
        // estimate tokens by fetching image dims
        if let Err(e) = estimate_image_tokens(&url, &mut session) { println!("Warning: could not estimate image tokens: {}", e); }
        let msgs = crate::extractor::image::extract_image(&url)?;
        review_and_send(msgs, &mut client, &model, &mut log, &mut session, true)?;
      }
      "sora_video" => {
        let prompt_text = Text::new("Enter video prompt:").prompt()?;
        let dur_input = Text::new("Duration (seconds)")
          .with_placeholder("5")
          .prompt()?;
        let duration: u32 = dur_input.trim().parse().unwrap_or(5);
        let fps_input = Text::new("FPS (frames per second)")
          .with_placeholder("24")
          .prompt()?;
        let fps: u32 = fps_input.trim().parse().unwrap_or(24);
        let size_options = vec![
          "1280x720",
          "720x1280",
          "896x512",
          "512x896",
          "640x352",
          "352x640",
        ];
        let res_input = Select::new("Size (resolution)", size_options.clone()).with_starting_cursor(0).prompt()?;
        // Show spinner while generating video
        let pb = crate::spinner::start("Generating Sora video...");
        let result: Result<std::path::PathBuf> = (|| {
          let key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow::anyhow!("Missing OPENAI_API_KEY in environment"))?;
          let openai = crate::api::clients::openai::OpenAIClient::new(key);
          let bytes = openai.generate_sora_video(&prompt_text, Some(duration), Some(fps), Some(res_input.trim()))?;
          let path = session.save_video_bytes(&prompt_text, "mp4", &bytes)?;
          Ok(path)
        })();
        crate::spinner::stop(&pb);
        match result {
          Ok(path) => {
            println!("\nSaved video to {}\n", path.to_string_lossy());
            session.append_message_to_file("\n\n***\n\n### User (Sora prompt):\n")?;
            session.append_message_to_file(&prompt_text)?;
            session.append_message_to_file("\n\n### Sora Video:\n")?;
            session.append_message_to_file(&format!("Saved to {}", path.to_string_lossy()))?;
          }
          Err(e) => {
            println!("\nFailed to generate Sora video: {}\n", e);
          }
        }
      }
      "pdf" => {
        let raw = Text::new("Enter path:").prompt()?;
        let path_str = normalize_path_input(&raw);
        let mut path_abs = resolve_local_or_download(&session, &path_str)?;
        // If no .pdf extension and a sibling with .pdf exists, use it
        if path_abs.extension().map(|e| e.to_string_lossy().to_lowercase()) != Some("pdf".into()) {
          let mut candidate = path_abs.clone();
          candidate.set_extension("pdf");
          if candidate.exists() { path_abs = candidate; }
        }
        if !path_abs.is_file() {
          println!("\nFile not found.\n");
          continue;
        }
        let get_images = Confirm::new("Get images?").with_default(false).prompt()?;
        session.append_message_to_file("\n\n***\n\n### User:\n")?;
        session.append_message_to_file(&path_abs.to_string_lossy())?;
        match crate::extractor::pdf::extract_pdf(&mut session, &path_abs, get_images) {
          Ok(msgs) => { review_and_send(msgs, &mut client, &model, &mut log, &mut session, true)?; }
          Err(e) => println!("\nFailed to extract pdf: {}\n", e),
        }
      }
      "xlsx" => {
        let raw = Text::new("Enter path:").prompt()?;
        let path_str = normalize_path_input(&raw);
        let path_abs = resolve_local_or_download(&session, &path_str)?;
        if !path_abs.is_file() {
          println!("\nFile not found.\n");
          continue;
        }
        session.append_message_to_file("\n\n***\n\n### User:\n")?;
        session.append_message_to_file(&path_abs.to_string_lossy())?;
        match crate::extractor::xlsx::extract_xlsx(&path_abs) {
          Ok(msgs) => { review_and_send(msgs, &mut client, &model, &mut log, &mut session, true)?; }
          Err(e) => println!("\nFailed to extract xlsx: {}\n", e),
        }
      }
      "git" => {
        let repo_url = Text::new("Enter git URL:").prompt()?;
        let repo_name = repo_url.split('/').next_back().unwrap_or("repo");
        let local_path = session.temp_dir.join(repo_name);
        if local_path.exists() { let _ = std::fs::remove_dir_all(&local_path); }
        match git2::Repository::clone(&repo_url, &local_path) {
          Ok(_) => {
            session.append_message_to_file("\n\n***\n\n### User:\n")?;
            session.append_message_to_file(&repo_url)?;
            match crate::extractor::dir::extract_dir(&session, &local_path, true) {
              Ok(msgs) => { review_and_send(msgs, &mut client, &model, &mut log, &mut session, true)?; }
              Err(e) => println!("\nFailed to extract git repo: {}\n", e),
            }
          }
          Err(e) => println!("\nFailed to clone repo: {}\n", e),
        }
      }
      _ => unreachable!(),
    }
  }

  Ok(())
}

// Remove any segments enclosed in <think>...</think> from the summary text.
fn strip_think_sections(input: &str) -> String {
  let mut out = String::with_capacity(input.len());
  let mut i = 0usize;
  let bytes = input.as_bytes();
  while i < input.len() {
    if let Some(start_rel) = input[i..].find("<think>") {
      let start = i + start_rel;
      // push text before the <think>
      out.push_str(&input[i..start]);
      let after_start = start + "<think>".len();
      if let Some(end_rel) = input[after_start..].find("</think>") {
        let end = after_start + end_rel + "</think>".len();
        // skip the think block entirely
        i = end;
        continue;
      } else {
        // No closing tag; drop the rest from the start tag
        break;
      }
    } else {
      // No more think tags; append remainder
      out.push_str(&input[i..]);
      break;
    }
  }
  // Ensure valid UTF-8 is preserved (we only sliced on char boundaries via &str)
  // But guard against accidental non-UTF indexing if this ever changes
  let _ = bytes; // silence unused warning if optimizations change
  out
}

fn handle_send(client: &mut Client, model: &str, log: &mut MessageLog, session: &mut ChatSession) -> Result<()> {
  // Show a spinner while the API call is in flight
  let pb = crate::spinner::start("Waiting for response...");
  let result = client.send_message(model, log);
  crate::spinner::stop(&pb);
  let response: ModelResponse = match result {
    Ok(r) => r,
    Err(e) => {
      println!("{} {}", "Error:".red().bold(), e.to_string().red());
      if matches!(std::env::var("RUST_BACKTRACE").ok().as_deref(), Some("true")) {
        println!("{}", "Stack trace:".bright_black());
        println!("{:#?}\n", e);
      } else {
        println!("{}", "(set RUST_BACKTRACE=1 to see a stack trace)".bright_black());
      }
      // Keep the app running after an API failure
      return Ok(());
    }
  };

  // Print the raw response object first
  if matches!(std::env::var("DEBUG_OUTPUT").ok().as_deref(), Some("true")) {
    let raw_filtered = response.raw.clone();
    print_colored_json(&raw_filtered);
  }

  // Add to log and display
  log.add_model(strip_think_sections(&response.text.clone()));
  session.append_message_to_file(&format!("\n\n### {}:\n", model))?;
  session.append_message_to_file(&strip_think_sections(&response.text))?;
  println!("\n{}:", model);
  let skin = termimad::MadSkin::default();
  skin.print_text(&strip_think_sections(&response.text));
  println!();
  Ok(())
}

fn print_colored_json(value: &serde_json::Value) {
  fn helper(v: &serde_json::Value, indent: usize, out: &mut String) {
    let pad = |n: usize| -> String { " ".repeat(n) };
    match v {
      serde_json::Value::Null => out.push_str(&format!("{}", "null".bright_black())),
      serde_json::Value::Bool(b) => out.push_str(&format!("{}", if *b { "true".magenta() } else { "false".magenta() })),
      serde_json::Value::Number(n) => out.push_str(&format!("{}", n.to_string().yellow())),
      serde_json::Value::String(s) => out.push_str(&format!("\"{}\"", s.green())),
      serde_json::Value::Array(arr) => {
        if arr.is_empty() { out.push_str("[]"); return; }
        out.push_str("[\n");
        for (i, item) in arr.iter().enumerate() {
          out.push_str(&pad(indent + 2));
          helper(item, indent + 2, out);
          if i + 1 != arr.len() { out.push(','); }
          out.push('\n');
        }
        out.push_str(&pad(indent));
        out.push(']');
      }
      serde_json::Value::Object(map) => {
        if map.is_empty() { out.push_str("{}"); return; }
        out.push_str("{\n");
        let len = map.len();
        for (idx, (k, val)) in map.iter().enumerate() {
          out.push_str(&pad(indent + 2));
          out.push_str(&format!("\"{}\"", k.bright_blue()));
          out.push_str(": ");
          helper(val, indent + 2, out);
          if idx + 1 != len { out.push(','); }
          out.push('\n');
        }
        out.push_str(&pad(indent));
        out.push('}');
      }
    }
  }
  let mut s = String::new();
  helper(value, 0, &mut s);
  println!("{}", s);
}

fn review_and_send(
  mut temp_msgs: Vec<Message>,
  client: &mut Client,
  model: &str,
  log: &mut MessageLog,
  session: &mut ChatSession,
  allow_directive: bool,
) -> Result<()> {
  // Optional directive to prepend (disabled for direct chat input)
  if allow_directive && Confirm::new("Add a directive?").with_default(true).prompt()? {
    let directive = Text::new("Enter a directive:").prompt()?;
    session.append_message_to_file("\n\n***\n\n### User:\n")?;
    session.append_message_to_file(&directive)?;
    temp_msgs.push(Message { role: Role::User, kind: MsgType::Text, content: directive });
  }

  // Token estimate for text only
  let text_tokens = estimate_text_tokens_for_msgs(&temp_msgs);
  let text_cost = (text_tokens as f64) / 1000.0 * 0.01;
  println!("This text data is ~{} tokens and ~${:.2}.", text_tokens, text_cost);

  // Image token count if maintained elsewhere
  if session.image_token_count > 0 {
    let img_cost = (session.image_token_count as f64) / 1000.0 * 0.01;
    println!("This image data is ~{} tokens and ~${:.2}.", session.image_token_count, img_cost);
  }

  if Confirm::new("Display the data?").with_default(false).prompt()? {
    let json = serde_json::to_string_pretty(&temp_msgs)?;
    println!("\n{}\n", json);
  }

  let proceed = Confirm::new("Do you want to send it?").with_default(true).prompt()?;
  if !proceed {
    println!("\nMessage not sent.");
    return Ok(());
  }

  // Extend message log then send
  log.extend(temp_msgs);
  handle_send(client, model, log, session)
}

fn estimate_text_tokens_for_msgs(msgs: &[Message]) -> usize {
  use tiktoken_rs::cl100k_base;
  let enc = cl100k_base().expect("load tokenizer");
  let text_only: Vec<&Message> = msgs.iter().filter(|m| matches!(m.kind, MsgType::Text)).collect();
  let s = serde_json::to_string(&text_only).unwrap_or_default();
  enc.encode_with_special_tokens(&s).len()
}

fn estimate_image_tokens(url: &str, session: &mut ChatSession) -> Result<()> {
  if !(url.starts_with("http://") || url.starts_with("https://")) { return Ok(()); }
  let client = crate::http::http_client()?;
  let resp = client.get(url).send()?;
  if !resp.status().is_success() { return Ok(()); }
  let bytes = resp.bytes()?;
  let reader = ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format();
  if let Ok(rdr) = reader {
    if let Ok(img) = rdr.decode() {
      let (w, h) = img.dimensions();
      if w >= 200 && h >= 200 {
        let tokens = count_image_tokens(w as usize, h as usize);
        session.image_token_count = tokens;
      }
    }
  }
  Ok(())
}

fn count_image_tokens(width: usize, height: usize) -> usize {
  let h = height.div_ceil(512); // ceil
  let w = width.div_ceil(512);
  let n = w * h;
  85 + 170 * n
}

fn normalize_path_input(s: &str) -> String {
  let t = s.trim();
  // Remove leading/trailing single or double quotes repeatedly
  let mut out = t.trim_matches(|c| c == '"' || c == '\'').to_string();
  // In case users double-quote twice, strip again
  out = out.trim_matches(|c| c == '"' || c == '\'').to_string();
  out
}

// Resolve a user-supplied path or URL to a local PathBuf.
// - For http/https URLs, downloads to the session temp dir.
// - For local paths, returns canonical path if possible, otherwise the original path.
fn resolve_local_or_download(session: &ChatSession, path_str: &str) -> Result<std::path::PathBuf> {
  if path_str.starts_with("http://") || path_str.starts_with("https://") {
    session.download_to_temp(path_str)
  } else {
    let path = std::path::Path::new(path_str);
    match std::fs::canonicalize(path) {
      Ok(p) => Ok(p),
      Err(_) => Ok(path.to_path_buf()),
    }
  }
}
