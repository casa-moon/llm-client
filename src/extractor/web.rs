use anyhow::{anyhow, Result};
use std::io::Cursor;
use image::{GenericImageView, ImageReader};
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};
use url::Url;

use crate::message_log::{Message, MsgType, Role};
use crate::session::ChatSession;

pub fn extract_text(session: &ChatSession, start_url: &str, depth: usize) -> Result<Vec<Message>> {
  let base = Url::parse(start_url).map_err(|e| anyhow!("Invalid URL: {}", e))?;
  let client = crate::http::http_client()?;
  let mut visited: HashSet<String> = HashSet::new();
  let mut queue: VecDeque<(Url, usize)> = VecDeque::new();
  queue.push_back((base.clone(), depth));
  let mut out: Vec<Message> = Vec::new();

  let text_sel = Selector::parse("p, h1, h2, h3, h4, h5, h6, span").unwrap();
  let link_sel = Selector::parse("a").unwrap();

  while let Some((url, d)) = queue.pop_front() {
    let key = url.as_str().to_string();
    if visited.contains(&key) { continue; }
    visited.insert(key.clone());

    let resp = client.get(url.as_str()).send();
    let Ok(resp) = resp else { continue };
    if !resp.status().is_success() { continue; }
    let body = match resp.text() {
      Ok(t) => t,
      Err(_) => continue
    };
    let doc = Html::parse_document(&body);

    let mut text = String::new();
    for el in doc.select(&text_sel) {
      let t = el.text().collect::<Vec<_>>().join(" ");
      if !t.trim().is_empty() {
        text.push_str(&t);
        text.push('\n');
      }
    }
    if !text.trim().is_empty() {
      let url_s = url.as_str().to_string();
      let _ = session.append_message_to_file(&format!("\n- {}\n", url_s));
      out.push(Message { role: Role::User, kind: MsgType::Text, content: url_s });
      out.push(Message { role: Role::User, kind: MsgType::Text, content: text });
    }

    if d > 0 {
      for a in doc.select(&link_sel) {
        if let Some(href) = a.value().attr("href") {
          if let Ok(next) = url.join(href) {
            if same_host(&base, &next) {
              queue.push_back((next, d - 1));
            }
          }
        }
      }
    }
  }

  Ok(out)
}

fn same_host(a: &Url, b: &Url) -> bool { a.domain() == b.domain() }

pub fn extract_images(session: &mut ChatSession, start_url: &str, depth: usize) -> Result<Vec<Message>> {
  let base = Url::parse(start_url).map_err(|e| anyhow!("Invalid URL: {}", e))?;
  let client = crate::http::http_client()?;
  let mut visited: HashSet<String> = HashSet::new();
  let mut queue: VecDeque<(Url, usize)> = VecDeque::new();
  queue.push_back((base.clone(), depth));
  let mut out: Vec<Message> = Vec::new();

  while let Some((url, d)) = queue.pop_front() {
    let key = url.as_str().to_string();
    if visited.contains(&key) { continue; }
    visited.insert(key.clone());

    let resp = client.get(url.as_str()).send();
    let Ok(resp) = resp else { continue };
    if !resp.status().is_success() { continue; }
    let body = match resp.text() {
      Ok(t) => t,
      Err(_) => continue
    };
    let doc = Html::parse_document(&body);

    let img_sel = Selector::parse("img").unwrap();
    let link_sel = Selector::parse("a").unwrap();

    let mut imgs: Vec<String> = Vec::new();
    for img in doc.select(&img_sel) {
      if let Some(src) = img.value().attr("src") {
        if let Ok(abs) = url.join(src) {
          let s = abs.as_str().to_string();
          if !imgs.contains(&s) && !s.contains("maps.googleapis.com") {
            imgs.push(s);
          }
        }
      }
    }

    for img_url in imgs {
      // avoid re-processing the same image URL
      if visited.contains(&img_url) { continue; }
      visited.insert(img_url.clone());

      // Attempt to fetch image and get dimensions
      if let Ok(resp) = client.get(&img_url).send() {
        if resp.status().is_success() {
          if let Ok(bytes) = resp.bytes() {
            // Skip SVG by quick heuristic
            if img_url.to_lowercase().ends_with(".svg") {
              // add as URL but no token count
              session.append_message_to_file(&format!("\n- {}\n", img_url)).ok();
              out.push(Message { role: Role::User, kind: MsgType::Text, content: img_url.clone() });
              out.push(Message { role: Role::User, kind: MsgType::Image, content: img_url.clone() });
              continue;
            }
            let reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format();
            if let Ok(rdr) = reader {
              if let Ok(img) = rdr.decode() {
                let (w, h) = img.dimensions();
                if w < 200 || h < 200 { continue; }
                // token estimate following JS heuristic
                let tokens = count_image_tokens(w as usize, h as usize);
                session.image_token_count += tokens;
                session.append_message_to_file(&format!("\n- {}\n", img_url)).ok();
                out.push(Message { role: Role::User, kind: MsgType::Text, content: img_url.clone() });
                out.push(Message { role: Role::User, kind: MsgType::Image, content: img_url.clone() });
              }
            }
          }
        }
      }
    }

    if d > 0 {
      for a in doc.select(&link_sel) {
        if let Some(href) = a.value().attr("href") {
          if let Ok(next) = url.join(href) {
            if same_host(&base, &next) {
              queue.push_back((next, d - 1));
            }
          }
        }
      }
    }
  }

  Ok(out)
}

fn count_image_tokens(width: usize, height: usize) -> usize {
  let h = (height + 511) / 512; // ceil
  let w = (width + 511) / 512;
  let n = w * h;
  85 + 170 * n
}
