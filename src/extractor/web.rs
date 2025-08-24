use anyhow::{anyhow, Context, Result};
use std::io::Cursor;
use image::{GenericImageView, ImageReader};
use scraper::{ElementRef, Html, Selector};
use std::collections::{HashSet, VecDeque};
use url::Url;

use crate::message_log::{Message, MsgType, Role};
use crate::session::ChatSession;
use crate::utils::count_image_tokens;

// Remove private-use glyphs, control chars (except newlines), and collapse whitespace.
fn clean_text(input: &str) -> String {
  let mut out = String::with_capacity(input.len());
  for ch in input.chars() {
    // Normalize NBSP to space
    let c = if ch == '\u{00A0}' { ' ' } else { ch };
    // Skip private-use area glyphs often used by icon fonts
    if is_private_use(c) { continue; }
    // Allow newline, convert other control chars to space
    if c.is_control() && c != '\n' { out.push(' '); continue; }
    out.push(c);
  }
  // Collapse whitespace sequences and trim per-line
  let collapsed = out
    .lines()
    .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
    .filter(|line| !line.trim().is_empty())
    .collect::<Vec<_>>()
    .join("\n");
  collapsed
}

fn is_private_use(c: char) -> bool {
  let u = c as u32;
  (0xE000..=0xF8FF).contains(&u) || (0xF0000..=0xFFFFD).contains(&u) || (0x100000..=0x10FFFD).contains(&u)
}

fn has_alnum(s: &str) -> bool { s.chars().any(|c| c.is_alphanumeric()) }

fn has_excluded_ancestor(el: &ElementRef, excluded_tags: &HashSet<&'static str>) -> bool {
  for a in el.ancestors() {
    if let Some(v) = a.value().as_element() {
      let name: &str = v.name.local.as_ref();
      if excluded_tags.contains(name) { return true; }
    }
  }
  false
}

// Common selectors and filters used by both basic and JS extractors.
struct CommonSelectors {
  primary_sel: Selector,
  container_sel: Selector,
  fallback_item_sel: Selector,
  link_sel: Selector,
  excluded: HashSet<&'static str>,
}

fn common_selectors() -> Result<CommonSelectors> {
  Ok(CommonSelectors {
    primary_sel: Selector::parse("article p, article h1, article h2, article h3, article h4, article h5, article h6, main p, main h1, main h2, main h3, main h4, main h5, main h6, p, h1, h2, h3, h4, h5, h6, blockquote, li")
      .map_err(|e| anyhow!(e.to_string()))?,
    container_sel: Selector::parse("main, article").map_err(|e| anyhow!(e.to_string()))?,
    fallback_item_sel: Selector::parse("p, h1, h2, h3, h4, h5, h6, li, blockquote")
      .map_err(|e| anyhow!(e.to_string()))?,
    link_sel: Selector::parse("a").map_err(|e| anyhow!(e.to_string()))?,
    excluded: [
      "header", "nav", "footer", "aside", "form", "script", "style", "noscript"
    ].into_iter().collect(),
  })
}

fn collect_text_lines(doc: &Html, sels: &CommonSelectors) -> Vec<String> {
  let mut lines: Vec<String> = Vec::new();
  for el in doc.select(&sels.primary_sel) {
    if has_excluded_ancestor(&el, &sels.excluded) { continue; }
    let t = el.text().collect::<Vec<_>>().join(" ");
    let t = clean_text(&t);
    if !t.is_empty() && has_alnum(&t) { lines.push(t); }
  }

  if lines.is_empty() {
    for container in doc.select(&sels.container_sel) {
      for el in container.select(&sels.fallback_item_sel) {
        if has_excluded_ancestor(&el, &sels.excluded) { continue; }
        let t = el.text().collect::<Vec<_>>().join(" ");
        let t = clean_text(&t);
        if !t.is_empty() && has_alnum(&t) { lines.push(t); }
      }
      if !lines.is_empty() { break; }
    }
  }

  lines
}

fn append_text_messages(session: &ChatSession, out: &mut Vec<Message>, url: &Url, text: &str) {
  if !text.trim().is_empty() {
    let url_s = url.as_str().to_string();
    let _ = session.append_message_to_file(&format!("- {}", url_s));
    out.push(Message { role: Role::User, kind: MsgType::Text, content: url_s });
    out.push(Message { role: Role::User, kind: MsgType::Text, content: text.to_string() });
  }
}

pub fn extract_text_basic(session: &ChatSession, start_url: &str, depth: usize) -> Result<Vec<Message>> {
  let base = Url::parse(start_url).map_err(|e| anyhow!("Invalid URL: {}", e))?;
  let client = crate::http::http_client()?;
  let mut visited: HashSet<String> = HashSet::new();
  let mut queue: VecDeque<(Url, usize)> = VecDeque::new();
  queue.push_back((base.clone(), depth));
  let mut out: Vec<Message> = Vec::new();
  let pb = crate::spinner::start("Extracting text (basic)...");
  let sels = common_selectors()?;

  while let Some((url, d)) = queue.pop_front() {
    pb.set_message(format!("Extracting {}", url));
    let key = url.as_str().to_string();
    if visited.contains(&key) { continue; }
    visited.insert(key.clone());

    let body = match client.get(url.as_str()).send() {
      Ok(resp) if resp.status().is_success() => match resp.text() { Ok(t) => t, Err(_) => continue },
      _ => continue,
    };
    let doc = Html::parse_document(&body);

    let lines = collect_text_lines(&doc, &sels);
    let text = lines.join("\n");
    append_text_messages(session, &mut out, &url, &text);

    enqueue_same_host_links(&base, &url, &doc, &sels.link_sel, d, &mut queue);
  }

  pb.finish_with_message(format!("Extracted {} page(s)", visited.len()));
  Ok(out)
}

// Treat only the exact host (and not the registrable domain) as the same host.
// Using `domain()` can return None for IPs and equate unrelated hosts (None == None),
// and also allows crossing subdomains (e.g., foo.example.com -> bar.example.com).
// Comparing `host_str()` prevents both issues.
fn same_host(a: &Url, b: &Url) -> bool {
  a.host_str() == b.host_str()
}

fn enqueue_same_host_links(
  base: &Url,
  current: &Url,
  doc: &Html,
  link_sel: &Selector,
  depth: usize,
  queue: &mut VecDeque<(Url, usize)>,
) {
  if depth > 0 {
    for a in doc.select(link_sel) {
      if let Some(href) = a.value().attr("href") {
        if let Ok(next) = current.join(href) {
          if same_host(base, &next) {
            queue.push_back((next, depth - 1));
          }
        }
      }
    }
  }
}

// Attempt to render pages with JavaScript using a headless Chrome tab.
// Returns an empty vec if Chrome can't be started or navigation fails.
fn extract_text_js_inner(session: &ChatSession, base: &Url, depth: usize) -> Result<Vec<Message>> {
  use headless_chrome::{Browser, LaunchOptionsBuilder};
  use std::time::Duration;

  let mut out: Vec<Message> = Vec::new();
  let pb = crate::spinner::start("Extracting text (JS)...");

  // Spin up headless Chrome (requires Chrome/Chromium installed)
  let launch_opts = LaunchOptionsBuilder::default()
    .headless(true)
    .build()
    .context("build chrome launch options")?;
  let browser = Browser::new(launch_opts).context("launch headless chrome")?;
  let tab = browser.new_tab().context("open new tab")?;

  let wait_ms: u64 = std::env::var("JS_RENDER_WAIT_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(1200);

  let mut visited: HashSet<String> = HashSet::new();
  let mut queue: VecDeque<(Url, usize)> = VecDeque::new();
  queue.push_back((base.clone(), depth));
  let sels = common_selectors()?;

  while let Some((url, d)) = queue.pop_front() {
    pb.set_message(format!("Rendering {}", url));
    let key = url.as_str().to_string();
    if visited.contains(&key) { continue; }
    visited.insert(key.clone());

    if let Err(e) = tab.navigate_to(url.as_str()) { eprintln!("[web:js] navigate error {}: {}", url, e); continue; }
    // Basic ready-state wait
    let _ = tab.wait_for_element("body");
    std::thread::sleep(Duration::from_millis(wait_ms));

    let content = match tab.get_content() {
      Ok(s) if !s.trim().is_empty() => s,
      _ => continue,
    };

    let doc = Html::parse_document(&content);
    let lines = collect_text_lines(&doc, &sels);
    let text = lines.join("\n");
    append_text_messages(session, &mut out, &url, &text);

    // Enqueue same-host links from the rendered DOM
    enqueue_same_host_links(base, &url, &doc, &sels.link_sel, d, &mut queue);
  }

  pb.finish_with_message(format!("Extracted {} page(s)", visited.len()));
  Ok(out)
}

pub fn extract_text_js(session: &ChatSession, start_url: &str, depth: usize) -> Result<Vec<Message>> {
  let base = Url::parse(start_url).map_err(|e| anyhow!("Invalid URL: {}", e))?;
  extract_text_js_inner(session, &base, depth)
}

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

    let img_sel = Selector::parse("img").map_err(|e| anyhow!(e.to_string()))?;
    let link_sel = Selector::parse("a").map_err(|e| anyhow!(e.to_string()))?;

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

    // Show a spinner while fetching images for this page
    let mut fetched = 0usize;
    let total = imgs.len();
    let spinner = if total > 0 {
      let s = crate::spinner::start(format!("Fetching {} images from {}", total, url));
      Some(s)
    } else { None };

    for img_url in imgs {
      // avoid re-processing the same image URL
      if visited.contains(&img_url) { continue; }
      visited.insert(img_url.clone());

      // Attempt to fetch image and get dimensions
      if let Some(s) = &spinner { s.set_message(format!("Fetching image {}/{}", fetched + 1, total)); }
      if let Ok(resp) = client.get(&img_url).send() {
        if resp.status().is_success() {
          let ct = resp.headers().get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_ascii_lowercase());
          if let Ok(bytes) = resp.bytes() {
            let is_svg = img_url.to_ascii_lowercase().ends_with(".svg")
              || ct.as_deref().map(|s| s.contains("image/svg+xml")).unwrap_or(false)
              || std::str::from_utf8(&bytes).map(|s| s.trim_start().starts_with("<svg")).unwrap_or(false);

            if is_svg {
              // Rasterize to PNG and embed as data URL
              if let Ok((data_url, w, h)) = crate::extractor::svg::rasterize_svg_to_png_b64(session, &bytes) {
                if w < 200 || h < 200 { continue; }
                let tokens = count_image_tokens(w as usize, h as usize);
                session.image_token_count += tokens;
                session.append_message_to_file(&format!("- {}", img_url)).ok();
                out.push(Message { role: Role::User, kind: MsgType::Image, content: data_url });
              } else {
                // Fallback: include original URL if rasterization fails
                session.append_message_to_file(&format!("- {}", img_url)).ok();
                out.push(Message { role: Role::User, kind: MsgType::Image, content: img_url.clone() });
              }
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
                session.append_message_to_file(&format!("- {}", img_url)).ok();
                out.push(Message { role: Role::User, kind: MsgType::Image, content: img_url.clone() });
              }
            }
          }
        }
      }
      fetched += 1;
    }

    if let Some(s) = spinner { s.finish_with_message(format!("Fetched {} images", fetched)); }

    enqueue_same_host_links(&base, &url, &doc, &link_sel, d, &mut queue);
  }

  Ok(out)
}
