use anyhow::Result;
use chrono::Utc;
use dirs::home_dir;
use std::fs;
use std::path::{Path, PathBuf};

pub struct ChatSession {
  pub dir: PathBuf,
  pub temp_dir: PathBuf,
  pub chat_file_dir: PathBuf,
  chat_file_path: PathBuf,
  pub image_token_count: usize,
}

impl ChatSession {
  pub fn new() -> Result<Self> {
    let is_termux = std::env::var("TERMUX_VERSION").is_ok();
    let home = if is_termux {
      PathBuf::from("/storage/emulated/0/Download")
    } else {
      home_dir().ok_or_else(|| anyhow::anyhow!("No home directory found"))?
    };
    let chatgpt_dir = if is_termux { "llm-client" } else { ".llm-client" };

    let files_dir = home.join(chatgpt_dir).join("files");
    let temp_dir = home.join(chatgpt_dir).join("temp");
    let chat_file_dir = home.join(chatgpt_dir).join("chats");

    fs::create_dir_all(&files_dir)?;
    fs::create_dir_all(&temp_dir)?;
    fs::create_dir_all(&chat_file_dir)?;

    let timestamp = Utc::now().to_rfc3339().replace(":", "-");
    let chat_file_name = format!("message-log-{}.md", timestamp);
    let chat_file_path = chat_file_dir.join(chat_file_name);

    Ok(Self {
      dir: files_dir,
      temp_dir,
      chat_file_dir,
      chat_file_path,
      image_token_count: 0,
    })
  }

  pub fn append_message_to_file(&self, message: &str) -> Result<()> {
    use std::io::Write;
    let mut f = fs::OpenOptions::new()
      .create(true)
      .append(true)
      .open(&self.chat_file_path)?;
    writeln!(f, "{}", message)?;
    Ok(())
  }

  pub fn clean_up(&self, save: bool) -> Result<()> {
    if self.chat_file_path.exists() && !save {
      let _ = fs::remove_file(&self.chat_file_path);
      println!("\nChat transcript deleted.\n");
    } else if self.chat_file_path.exists() && save {
      println!("\nChat transcript saved to {}\n", self.chat_file_path.to_string_lossy());
    }
    if self.temp_dir.exists() {
      let _ = fs::remove_dir_all(&self.temp_dir);
    }
    Ok(())
  }

  // Rename the chat file to include a short summary slug of the conversation.
  // If `summary_opt` is None, it derives a summary from the file contents.
  // Returns the new path (or original if unchanged).
  pub fn rename_with_summary(&mut self, summary_opt: Option<&str>) -> Result<PathBuf> {
    if !self.chat_file_path.exists() { return Ok(self.chat_file_path.clone()); }

    let owned;
    let summary: &str = match summary_opt {
      Some(s) if !s.trim().is_empty() => s,
      _ => {
        owned = summarize_chat_for_slug(&std::fs::read_to_string(&self.chat_file_path).unwrap_or_default());
        if owned.is_empty() { return Ok(self.chat_file_path.clone()); }
        &owned
      }
    };

    let slug = to_slug(summary);
    if slug.is_empty() { return Ok(self.chat_file_path.clone()); }
    let new_name = format!("{}.md", slug);
    let mut new_path = self.chat_file_dir.join(&new_name);

    if new_path.exists() {
      let mut i = 2;
      loop {
        let candidate = self.chat_file_dir.join(format!("{}-{}.md", slug, i));
        if !candidate.exists() {
          new_path = candidate;
          break;
        }
        i += 1;
        if i > 50 { break; }
      }
    }

    if new_path != self.chat_file_path { std::fs::rename(&self.chat_file_path, &new_path)?; self.chat_file_path = new_path.clone(); }
    Ok(self.chat_file_path.clone())
  }

  pub fn copy_file_to_dir<P: AsRef<Path>>(&self, src: P) -> Result<PathBuf> {
    let src_path = src.as_ref();
    let dest = self.dir.join(
      src_path.file_name().ok_or_else(|| anyhow::anyhow!("Invalid file name"))?,
    );
    if src_path != dest {
      fs::copy(src_path, &dest)?;
      println!("File copied to {} directory.", self.dir.to_string_lossy());
    }
    Ok(dest)
  }

  pub fn download_to_temp(&self, url: &str) -> Result<PathBuf> {
    use reqwest::blocking::Client;
    use std::io::Write;
    let client = Client::builder().build()?;
    let resp = client.get(url).send()?;
    if !resp.status().is_success() {
      return Err(anyhow::anyhow!("Failed to download: {}", resp.status()));
    }
    let filename = url
      .split('/')
      .last()
      .filter(|s| !s.is_empty())
      .unwrap_or("downloaded.file");
    let path = self.temp_dir.join(filename);
    let mut file = fs::File::create(&path)?;
    let bytes = resp.bytes()?;
    file.write_all(&bytes)?;
    Ok(path)
  }
}

fn summarize_chat_for_slug(markdown: &str) -> String {
  let mut in_user = false;
  let mut collected = String::new();
  for line in markdown.lines() {
    if line.trim_start().starts_with("### ") {
      in_user = line.contains("User:");
      if !in_user && !collected.is_empty() { break; }
      continue;
    }
    if in_user {
      let t = line.trim();
      if t.starts_with("```") { continue; }
      if t.is_empty() { continue; }
      if collected.len() + t.len() + 1 > 240 { break; }
      if !collected.is_empty() { collected.push(' '); }
      collected.push_str(t);
      if t.ends_with('.') || t.ends_with('!') || t.ends_with('?') { break; }
    }
  }
  if collected.is_empty() {
    for line in markdown.lines() {
      let t = line.trim();
      if t.is_empty() { continue; }
      if t.starts_with("### ") || t.starts_with("# ") { return t.trim_matches('#').trim().to_string(); }
      return t.to_string();
    }
  }
  collected
}

fn to_slug(s: &str) -> String {
  let mut out = String::new();
  let s = s.to_lowercase();
  let mut last_dash = false;
  let mut words = 0usize;
  for ch in s.chars() {
    if ch.is_ascii_alphanumeric() {
      out.push(ch);
      last_dash = false;
    } else if ch.is_whitespace() || matches!(ch, '-' | '_' | '/' | ':') {
      if !last_dash && !out.is_empty() {
        out.push('-');
        last_dash = true;
        words += 1;
        if words >= 8 { break; }
      }
    }
    if out.len() >= 48 { break; }
  }
  out.trim_matches('-').to_string()
}
