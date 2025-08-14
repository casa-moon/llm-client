use anyhow::{anyhow, Result};
use base64::Engine;

use crate::message_log::{Message, MsgType, Role};
use crate::session::ChatSession;

pub fn extract_pdf(session: &mut ChatSession, path: &std::path::Path, get_images: bool) -> Result<Vec<Message>> {
  if !path.exists() { return Err(anyhow!("File not found: {}", path.display())); }

  let pb = crate::spinner::start("Extracting PDF text...");
  let text = pdf_extract::extract_text(path).map_err(|e| anyhow!("Failed to extract text from PDF: {}", e))?;
  crate::spinner::stop(&pb);
  
  let mut out = Vec::new();
  out.push(Message { role: Role::User, kind: MsgType::Text, content: text });
  if get_images {
    // Try extracting embedded images (DCTDecode only)
    let pb = crate::spinner::start("Extracting PDF images...");
    match extract_pdf_images_dct(session, path) {
      Ok(mut imgs) => {
        crate::spinner::stop(&pb);
        for (label, data_url, tokens) in imgs.drain(..) {
          out.push(Message { role: Role::User, kind: MsgType::Text, content: label });
          out.push(Message { role: Role::User, kind: MsgType::Image, content: data_url });
          session.image_token_count += tokens;
        }
      }
      Err(e) => {
        crate::spinner::stop(&pb);
        out.push(Message { role: Role::User, kind: MsgType::Text, content: format!("[Failed to extract PDF images: {}]", e) });
      }
    }
  }
  Ok(out)
}

fn extract_pdf_images_dct(session: &mut ChatSession, path: &std::path::Path) -> Result<Vec<(String, String, usize)>> {
  use lopdf::{Document, Object};
  use image::{ImageBuffer, RgbImage, Luma};
  use image::codecs::jpeg::JpegEncoder;
  use flate2::read::ZlibDecoder;
  use std::io::Read;
  let doc = Document::load(path)?;
  let mut out: Vec<(String, String, usize)> = Vec::new();
  let mut image_idx = 1usize;

  for (_obj_id, obj) in &doc.objects {
    if let Object::Stream(stream) = obj {
      let dict = &stream.dict;
      // Check /Subtype /Image
      if let Ok(Object::Name(subtype)) = dict.get(b"Subtype") { if subtype != b"Image" { continue; } } else { continue; }

      // width/height
      let width = dict.get(b"Width").and_then(|o| o.as_i64()).unwrap_or(0) as usize;
      let height = dict.get(b"Height").and_then(|o| o.as_i64()).unwrap_or(0) as usize;
      if width == 0 || height == 0 { continue; }

      // Detect filters
      let filter_is_dct = match dict.get(b"Filter").ok() {
        Some(Object::Name(name)) => name == b"DCTDecode",
        Some(Object::Array(arr)) => arr.iter().any(|o| matches!(o, Object::Name(n) if n == b"DCTDecode")),
        _ => false,
      };
      let filter_is_flate = match dict.get(b"Filter").ok() {
        Some(Object::Name(name)) => name == b"FlateDecode",
        Some(Object::Array(arr)) => arr.iter().any(|o| matches!(o, Object::Name(n) if n == b"FlateDecode")),
        _ => false,
      };
      let filter_is_jpx = match dict.get(b"Filter").ok() {
        Some(Object::Name(name)) => name == b"JPXDecode",
        Some(Object::Array(arr)) => arr.iter().any(|o| matches!(o, Object::Name(n) if n == b"JPXDecode")),
        _ => false,
      };

      if filter_is_dct {
        // Raw JPEG data in the stream
        let data = &stream.content;
        // Save a copy in temp for reference
        let file_path = session.temp_dir.join(format!("image{}.jpeg", image_idx));
        std::fs::write(&file_path, data)?;

        // Count tokens and build data URL
        let tokens = count_image_tokens(width, height);
        let b64 = base64::engine::general_purpose::STANDARD.encode(data);
        let data_url = format!("data:image/jpeg;base64,{}", b64);

        session.append_message_to_file(&format!("\n- {}\n", file_path.display())).ok();
        out.push((format!("image{}", image_idx), data_url, tokens));
        image_idx += 1;
      } else if filter_is_flate {
        // FlateDecode with or without PNG predictors
        let bpc = dict.get(b"BitsPerComponent").and_then(|o| o.as_i64()).unwrap_or(8) as usize;
        if bpc != 8 { continue; }
        let channels = match dict.get(b"ColorSpace").ok() {
          Some(Object::Name(cs)) if cs == b"DeviceRGB" => 3usize,
          Some(Object::Name(cs)) if cs == b"DeviceGray" => 1usize,
          Some(Object::Name(cs)) if cs == b"DeviceCMYK" => 4usize,
          Some(Object::Array(arr)) if arr.first().and_then(|o| o.as_name().ok()) == Some(b"ICCBased") => 3usize,
          _ => continue,
        };
        let mut decoder = ZlibDecoder::new(&stream.content[..]);
        let mut raw = Vec::new();
        if decoder.read_to_end(&mut raw).is_err() { continue; }

        // If DecodeParms with PNG predictor present, unfilter rows
        let mut data = raw;
        if let Ok(Object::Dictionary(dp)) = dict.get(b"DecodeParms") {
          let predictor = dp.get(b"Predictor").and_then(|o| o.as_i64()).unwrap_or(1) as i64;
          if predictor == 12 || predictor == 10 || predictor == 11 || predictor == 13 || predictor == 14 || predictor == 15 {
            let colors = dp.get(b"Colors").and_then(|o| o.as_i64()).unwrap_or(channels as i64) as usize;
            let cols = dp.get(b"Columns").and_then(|o| o.as_i64()).unwrap_or(width as i64) as usize;
            let bpc_dp = dp.get(b"BitsPerComponent").and_then(|o| o.as_i64()).unwrap_or(bpc as i64) as usize;
            if bpc_dp != 8 { continue; }
            if colors != channels { /* assume given colors */ }
            if let Some(unfiltered) = png_unfilter(&data, cols, colors) { data = unfiltered; } else { continue; }
          }
        }

        let expected = width * height * channels;
        if data.len() < expected { continue; }
        data.truncate(expected);

        let mut jpeg_buf: Vec<u8> = Vec::new();
        if channels == 4 {
          // Convert CMYK to RGB
          let mut rgb_data = Vec::with_capacity(width * height * 3);
          for i in (0..data.len()).step_by(4) {
            if i + 3 >= data.len() { break; }
            let c = data[i] as f32 / 255.0;
            let m = data[i + 1] as f32 / 255.0;
            let y = data[i + 2] as f32 / 255.0;
            let k = data[i + 3] as f32 / 255.0;
            let r = (1.0 - (c * (1.0 - k) + k)) * 255.0;
            let g = (1.0 - (m * (1.0 - k) + k)) * 255.0;
            let b = (1.0 - (y * (1.0 - k) + k)) * 255.0;
            rgb_data.push(r.clamp(0.0, 255.0) as u8);
            rgb_data.push(g.clamp(0.0, 255.0) as u8);
            rgb_data.push(b.clamp(0.0, 255.0) as u8);
          }
          if let Some(rgb) = RgbImage::from_raw(width as u32, height as u32, rgb_data) {
            let dynimg = image::DynamicImage::ImageRgb8(rgb);
            let mut enc = JpegEncoder::new(&mut jpeg_buf);
            if enc.encode_image(&dynimg).is_err() { continue; }
          } else { continue; }
        } else if channels == 3 {
          if let Some(rgb) = RgbImage::from_raw(width as u32, height as u32, data) {
            let dynimg = image::DynamicImage::ImageRgb8(rgb);
            let mut enc = JpegEncoder::new(&mut jpeg_buf);
            if enc.encode_image(&dynimg).is_err() { continue; }
          } else { continue; }
        } else {
          if let Some(luma) = ImageBuffer::<Luma<u8>, _>::from_raw(width as u32, height as u32, data) {
            let dynimg = image::DynamicImage::ImageLuma8(luma);
            let mut enc = JpegEncoder::new(&mut jpeg_buf);
            if enc.encode_image(&dynimg).is_err() { continue; }
          } else { continue; }
        }

        let file_path = session.temp_dir.join(format!("image{}.jpeg", image_idx));
        std::fs::write(&file_path, &jpeg_buf)?;
        let tokens = count_image_tokens(width, height);
        let b64 = base64::engine::general_purpose::STANDARD.encode(&jpeg_buf);
        let data_url = format!("data:image/jpeg;base64,{}", b64);
        session.append_message_to_file(&format!("\n- {}\n", file_path.display())).ok();
        out.push((format!("image{}", image_idx), data_url, tokens));
        image_idx += 1;
      } else if filter_is_jpx {
        // JPEG2000 codestream or JP2; we embed as data URL and use dict width/height for token estimate
        let data = &stream.content;
        let file_path = session.temp_dir.join(format!("image{}.jp2", image_idx));
        std::fs::write(&file_path, data)?;
        let tokens = count_image_tokens(width, height);
        let b64 = base64::engine::general_purpose::STANDARD.encode(data);
        // Most browsers/providers recognize image/jp2
        let data_url = format!("data:image/jp2;base64,{}", b64);
        session.append_message_to_file(&format!("\n- {}\n", file_path.display())).ok();
        out.push((format!("image{}", image_idx), data_url, tokens));
        image_idx += 1;
      } else {
        continue;
      }
    }
  }
  Ok(out)
}

fn png_unfilter(data: &[u8], cols: usize, colors: usize) -> Option<Vec<u8>> {
  // Each scanline: filter byte + bytes; stride = colors
  let bpp = colors; // since bpc==8
  let row_bytes = cols * bpp;
  let mut i = 0usize;
  let mut out: Vec<u8> = Vec::with_capacity(data.len());
  let mut prev: Vec<u8> = vec![0u8; row_bytes];
  while i < data.len() {
    let filter = *data.get(i)?;
    i += 1;
    if i + row_bytes > data.len() { return None; }
    let mut row = vec![0u8; row_bytes];
    let src = &data[i..i + row_bytes];
    match filter {
      0 => { row.copy_from_slice(src); }
      1 => { // Sub
        for x in 0..row_bytes {
          let left = if x >= bpp { row[x - bpp] } else { 0 };
          row[x] = src[x].wrapping_add(left);
        }
      }
      2 => { // Up
        for x in 0..row_bytes { row[x] = src[x].wrapping_add(prev[x]); }
      }
      3 => { // Average
        for x in 0..row_bytes {
          let left = if x >= bpp { row[x - bpp] } else { 0 };
          let up = prev[x];
          row[x] = src[x].wrapping_add(((left as u16 + up as u16) / 2) as u8);
        }
      }
      4 => { // Paeth
        for x in 0..row_bytes {
          let a = if x >= bpp { row[x - bpp] } else { 0 } as i16;
          let b = prev[x] as i16;
          let c = if x >= bpp { prev[x - bpp] } else { 0 } as i16;
          let p = a + b - c;
          let pa = (p - a).abs();
          let pb = (p - b).abs();
          let pc = (p - c).abs();
          let pr = if pa <= pb && pa <= pc { a } else if pb <= pc { b } else { c } as i16;
          row[x] = src[x].wrapping_add(pr as u8);
        }
      }
      _ => return None,
    }
    out.extend_from_slice(&row);
    prev.copy_from_slice(&row);
    i += row_bytes;
  }
  Some(out)
}

fn count_image_tokens(width: usize, height: usize) -> usize {
  let h = (height + 511) / 512; // ceil
  let w = (width + 511) / 512;
  let n = w * h;
  85 + 170 * n
}
