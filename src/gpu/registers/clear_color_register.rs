use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ClearColorRegister {
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub fog: bool,
  pub alpha: u8,
  pub clear_polygon_id: u8
}

impl ClearColorRegister {
  pub fn new() -> Self {
    Self {
      r: 0,
      g: 0,
      b: 0,
      fog: false,
      alpha: 0,
      clear_polygon_id: 0
    }
  }

  pub fn write(&mut self, value: u32) {
    self.r = (value & 0x1f) as u8;
    self.g = (value >> 5 & 0x1f) as u8;
    self.b = (value >> 10 & 0x1f) as u8;

    self.fog = (value >> 15) & 0b1 == 1;
    self.alpha = ((value >> 16) & 0x1f) as u8;
    self.clear_polygon_id = ((value >> 24) & 0x7f) as u8;
  }
}