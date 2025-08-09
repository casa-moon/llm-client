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
    let chatgpt_dir = if is_termux { "chatgpt-client" } else { ".chatgpt-client" };

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
      println!(
        "\nChat transcript saved to {}\n",
        self.chat_file_path.to_string_lossy()
      );
    }
    if self.temp_dir.exists() {
      let _ = fs::remove_dir_all(&self.temp_dir);
    }
    Ok(())
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
