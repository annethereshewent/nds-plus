
pub fn read_word(bytes: &Vec<u8>, offset: usize) -> u32 {
  (bytes[offset] as u32) | (bytes[offset + 1] as u32) << 8 | (bytes[offset + 2] as u32) << 16 | (bytes[offset + 3] as u32) << 24
}