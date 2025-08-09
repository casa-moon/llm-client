use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

use crate::message_log::{Message, MsgType, Role};

pub fn extract_text<P: AsRef<Path>>(path: P) -> Result<Vec<Message>> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(anyhow!("File not found: {}", path.display()));
    }

    let bytes = fs::read(path)?;
    let content = match String::from_utf8(bytes) {
        Ok(s) => s,
        Err(_) => return Err(anyhow!("File is not valid UTF-8: {}", path.display())),
    };

    Ok(vec![Message { role: Role::User, kind: MsgType::Text, content }])
}
