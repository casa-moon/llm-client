pub fn count_image_tokens(width: usize, height: usize) -> usize {
  let h = height.div_ceil(512);
  let w = width.div_ceil(512);
  let n = w * h;
  85 + 170 * n
}