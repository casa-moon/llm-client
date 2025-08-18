use anyhow::Result;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::extractor::file as file_extractor;
use crate::message_log::{Message, MsgType, Role};
use crate::session::ChatSession;

pub fn extract_dir(session: &ChatSession, path: &Path, recursive: bool) -> Result<Vec<Message>> {
  let mut out: Vec<Message> = Vec::new();
  let excluded: HashSet<&str> = [
    "node_modules",
    "temp",
    "archive",
    "ideas",
    "charts",
    "chats",
    "package-lock.json",
    "Cargo.lock",
    ".idea",
    ".env",
    ".gitignore",
    ".git",
    ".terraform",
    ".editorconfig",
    ".pre-commit-config.yaml",
    ".releaserc.json",
    "CHANGELOG.md",
    "LICENSE",
    "target"
  ]
    .into_iter()
    .collect();

  if recursive {
    // Use filter_entry to prune excluded directories from traversal
    for entry in WalkDir::new(path)
      .into_iter()
      .filter_entry(|e| {
        // Always include the root; apply exclusions to children
        if e.depth() == 0 { return true; }
        let name = e.file_name().to_str().unwrap_or("");
        !excluded.contains(name)
      })
      .filter_map(|e| e.ok())
    {
      let p = entry.path();
      if p.is_dir() { continue; }
      if p.is_file() {
        match file_extractor::extract_text(p) {
          Ok(msgs) => {
            // add file path then its contents
            out.push(Message { role: Role::User, kind: MsgType::Text, content: p.display().to_string() });
            out.extend(msgs);
            let _ = session.append_message_to_file(&format!("- {}", p.display()));
          }
          Err(_) => continue,
        }
      }
    }
  } else {
    if path.is_dir() {
      for entry in fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();
        let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
        if excluded.contains(name.as_str()) { continue; }
        if p.is_file() {
          if let Ok(msgs) = file_extractor::extract_text(&p) {
            out.push(Message { role: Role::User, kind: MsgType::Text, content: p.display().to_string() });
            out.extend(msgs);
            let _ = session.append_message_to_file(&format!("\n- {}\n", p.display()));
          }
        }
      }
    }
  }

  Ok(out)
}
