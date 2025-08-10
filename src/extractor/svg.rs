use anyhow::{anyhow, Result};
use base64::Engine;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::session::ChatSession;

// Rasterize SVG bytes to PNG and return (data_url, width, height)
pub fn rasterize_svg_to_png_b64(session: &ChatSession, svg_bytes: &[u8]) -> Result<(String, u32, u32)> {
  // Parse SVG
  let opt = usvg::Options::default();
  // Keep defaults; no external resources fetching
  let tree = usvg::Tree::from_data(svg_bytes, &opt)
    .map_err(|_| anyhow!("Failed to parse SVG"))?;

  let size = tree.size().to_int_size();
  let (w, h) = (size.width() as u32, size.height() as u32);
  if w == 0 || h == 0 { return Err(anyhow!("Invalid SVG size")); }

  // Render with resvg into a pixmap
  let mut pixmap = tiny_skia::Pixmap::new(w, h).ok_or_else(|| anyhow!("Failed to create pixmap"))?;
  let mut pmut = pixmap.as_mut();
  let transform = tiny_skia::Transform::identity();
  resvg::render(&tree, transform, &mut pmut);

  // Save to a temp PNG file using tiny-skia PNG feature
  let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
  let file_path = session.temp_dir.join(format!("websvg_{}.png", ts));
  pixmap.save_png(&file_path).map_err(|e| anyhow!("Failed to save PNG: {}", e))?;

  // Read, base64-encode, and wrap as data URL
  let png_bytes = std::fs::read(&file_path)?;
  let b64 = base64::engine::general_purpose::STANDARD.encode(png_bytes);
  let data_url = format!("data:image/png;base64,{}", b64);

  Ok((data_url, w, h))
}
