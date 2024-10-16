use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SpecularColor {
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub shininess_table_enable: bool
}

impl SpecularColor {
  pub fn new() -> Self {
    Self {
      r: 0,
      g: 0,
      b: 0,
      shininess_table_enable: false
    }
  }

  pub fn write(&mut self, value: u16) {
    self.r = (value & 0x1f) as u8;
    self.g = ((value >> 5) & 0x1f) as u8;
    self.b = ((value >> 10) & 0x1f) as u8;
    self.shininess_table_enable = (value >> 15) & 0b1 == 1;
  }
}