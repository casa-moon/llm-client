use anyhow::Result;

use crate::message_log::{Message, MsgType, Role};

pub fn extract_image(url: &str) -> Result<Vec<Message>> {
  if url.starts_with("http://") || url.starts_with("https://") {
    Ok(vec![
      Message { role: Role::User, kind: MsgType::Text, content: url.to_string() },
      Message { role: Role::User, kind: MsgType::Image, content: url.to_string() },
    ])
  } else {
    // Only http(s) supported currently
    Ok(vec![])
  }
}

