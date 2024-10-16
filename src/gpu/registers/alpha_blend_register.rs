use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AlphaBlendRegister {
  pub eva: u8,
  pub evb: u8
}

impl AlphaBlendRegister {
  pub fn new() -> Self {
    Self {
      eva: 0,
      evb: 0
    }
  }

  pub fn write(&mut self, value: u16) {
    self.eva = (value & 0x1f).min(16) as u8;
    self.evb = ((value >> 8) & 0x1f ).min(16) as u8;
  }

  pub fn read(&self) -> u16 {
    self.eva as u16 | (self.evb as u16) << 8
  }
}