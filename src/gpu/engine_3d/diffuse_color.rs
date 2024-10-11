use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DiffuseColor {
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub set_vertex_color: bool
}

impl DiffuseColor {
  pub fn new() -> Self {
    Self {
      r: 0,
      g: 0,
      b: 0,
      set_vertex_color: false
    }
  }

  pub fn write(&mut self, value: u16) {
    self.r = (value & 0x1f) as u8;
    self.g = ((value >> 5) & 0x1f) as u8;
    self.b = ((value >> 10) & 0x1f) as u8;
    self.set_vertex_color = (value >> 15) & 0b1 == 1;
  }
}