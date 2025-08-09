use anyhow::{anyhow, Result};
use calamine::{open_workbook_auto, Reader, DataType};

use crate::message_log::{Message, MsgType, Role};

pub fn extract_xlsx(path: &std::path::Path) -> Result<Vec<Message>> {
  if !path.exists() { return Err(anyhow!("File not found: {}", path.display())); }
  let mut wb = open_workbook_auto(path).map_err(|e| anyhow!("Failed to open xlsx: {}", e))?;
  let mut out: Vec<Message> = Vec::new();
  for name in wb.sheet_names().to_owned() {
    if let Some(Ok(range)) = wb.worksheet_range(&name) {
      let mut buf = String::new();
      for row in range.rows() {
        let mut first = true;
        for cell in row.iter() {
          if !first { buf.push(','); }
          first = false;
          let s = match cell {
            DataType::Empty => String::new(),
            DataType::Float(f) => f.to_string(),
            DataType::Int(i) => i.to_string(),
            DataType::Bool(b) => b.to_string(),
            DataType::String(s) => s.clone(),
            DataType::DateTime(f) => f.to_string(),
            _ => String::new(),
          };
          buf.push_str(&s);
        }
        buf.push('\n');
      }
      if !buf.trim().is_empty() {
        out.push(Message { role: Role::User, kind: MsgType::Text, content: format!("Sheet: {}", name) });
        out.push(Message { role: Role::User, kind: MsgType::Text, content: buf });
      }
    }
  }
  Ok(out)
}

