use anyhow::Result;
use inquire::{Confirm, Select, Text};

use crate::api::{create_client, API_CHOICES, Client};
use crate::message_log::{Message, MessageLog, MsgType, Role};
use image::{GenericImageView, ImageReader};
use crate::session::ChatSession;
use crate::extractor::file as file_extractor;
use crate::markdown;

pub fn run_app() -> Result<()> {
  // choose API
  let choices: Vec<String> = API_CHOICES
    .iter()
    .map(|c| format!("{}", c.label))
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
    ("Direct Chat Input", "chat"),
    ("Multi-line Input", "multi"),
    ("File", "file"),
    ("Directory", "dir"),
    ("Web Page", "web"),
    ("Image", "image"),
    ("PDF", "pdf"),
    ("Excel (xlsx)", "xlsx"),
    ("Git Repository", "git"),
    ("Save Chat Session", "save"),
    ("Exit Application", "exit"),
    // Future: pdf/xlsx/image/dir/git/web
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
        review_and_send(temp_msgs, &mut client, &model, &mut log, &mut session)?;
      }
      "multi" => {
        let input = match dialoguer::Editor::new().require_save(false).edit("\n# Enter multi-line input below\n") {
          Ok(Some(s)) => s,
          _ => Text::new("Multi-line input:").prompt()?,
        };
        session.append_message_to_file("\n\n***\n\n### User:\n")?;
        session.append_message_to_file(&input)?;
        let temp_msgs = vec![Message { role: Role::User, kind: MsgType::Text, content: input }];
        review_and_send(temp_msgs, &mut client, &model, &mut log, &mut session)?;
      }
      "file" => {
        let path_str = Text::new("Enter path:").prompt()?;
        let path_abs = if path_str.starts_with("http://") || path_str.starts_with("https://") {
          session.download_to_temp(&path_str)?
        } else {
          let path = std::path::Path::new(&path_str);
          match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => path.to_path_buf()
          }
        };
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
            review_and_send(msgs, &mut client, &model, &mut log, &mut session)?;
          }
          Err(e) => {
            println!("\nFailed to extract file: {}\n", e);
          }
        }
      }
      "dir" => {
        let path_str = Text::new("Enter path:").prompt()?;
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
          Ok(msgs) => { review_and_send(msgs, &mut client, &model, &mut log, &mut session)?; }
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
        review_and_send(total, &mut client, &model, &mut log, &mut session)?;
      }
      "image" => {
        let url = Text::new("Enter image URL:").prompt()?;
        session.append_message_to_file("\n\n***\n\n### User:\n")?;
        session.append_message_to_file(&url)?;
        // estimate tokens by fetching image dims
        if let Err(e) = estimate_image_tokens(&url, &mut session) { println!("Warning: could not estimate image tokens: {}", e); }
        let msgs = crate::extractor::image::extract_image(&url)?;
        review_and_send(msgs, &mut client, &model, &mut log, &mut session)?;
      }
      "pdf" => {
        let path_str = Text::new("Enter path:").prompt()?;
        let mut path_abs = if path_str.starts_with("http://") || path_str.starts_with("https://") {
          session.download_to_temp(&path_str)?
        } else { std::fs::canonicalize(&path_str).unwrap_or_else(|_| std::path::PathBuf::from(&path_str)) };
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
          Ok(msgs) => { review_and_send(msgs, &mut client, &model, &mut log, &mut session)?; }
          Err(e) => println!("\nFailed to extract pdf: {}\n", e),
        }
      }
      "xlsx" => {
        let path_str = Text::new("Enter path:").prompt()?;
        let path_abs = if path_str.starts_with("http://") || path_str.starts_with("https://") {
          session.download_to_temp(&path_str)?
        } else { std::fs::canonicalize(&path_str).unwrap_or_else(|_| std::path::PathBuf::from(&path_str)) };
        if !path_abs.is_file() {
          println!("\nFile not found.\n");
          continue;
        }
        session.append_message_to_file("\n\n***\n\n### User:\n")?;
        session.append_message_to_file(&path_abs.to_string_lossy())?;
        match crate::extractor::xlsx::extract_xlsx(&path_abs) {
          Ok(msgs) => { review_and_send(msgs, &mut client, &model, &mut log, &mut session)?; }
          Err(e) => println!("\nFailed to extract xlsx: {}\n", e),
        }
      }
      "git" => {
        let repo_url = Text::new("Enter git URL:").prompt()?;
        let repo_name = repo_url.split('/').last().unwrap_or("repo");
        let local_path = session.temp_dir.join(repo_name);
        if local_path.exists() { let _ = std::fs::remove_dir_all(&local_path); }
        match git2::Repository::clone(&repo_url, &local_path) {
          Ok(_) => {
            session.append_message_to_file("\n\n***\n\n### User:\n")?;
            session.append_message_to_file(&repo_url)?;
            match crate::extractor::dir::extract_dir(&session, &local_path, true) {
              Ok(msgs) => { review_and_send(msgs, &mut client, &model, &mut log, &mut session)?; }
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

fn handle_send(client: &mut Client, model: &str, log: &mut MessageLog, session: &mut ChatSession) -> Result<()> {
  // In JS, there's a confirmation step; skip for initial port
  let response = client.send_message(model, &log)?;

  // Add to log and display
  log.add_model(response.clone());
  session.append_message_to_file("\n\n### Assistant:\n")?;
  session.append_message_to_file(&response)?;
  let rendered = markdown::render(&response);
  println!("{}", rendered);
  Ok(())
}

fn review_and_send(mut temp_msgs: Vec<Message>, client: &mut Client, model: &str, log: &mut MessageLog, session: &mut ChatSession) -> Result<()> {
  // Optional directive to prepend
  if Confirm::new("Add a directive?").with_default(false).prompt()? {
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
  let client = reqwest::blocking::Client::new();
  let resp = client.get(url).send()?;
  if !resp.status().is_success() { return Ok(()); }
  let bytes = resp.bytes()?;
  let reader = ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format();
  if let Ok(mut rdr) = reader {
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
  let h = (height + 511) / 512; // ceil
  let w = (width + 511) / 512;
  let n = w * h;
  85 + 170 * n
}
