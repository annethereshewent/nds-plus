use std::cmp;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct BrightnessRegister {
  pub evy: u8
}

impl BrightnessRegister {
  pub fn new() -> Self {
    Self {
      evy: 0
    }
  }

  pub fn write(&mut self, value: u16) {
    self.evy = cmp::min(16, (value & 0b11111) as u8);
  }

  pub fn read(&self) -> u16 {
    self.evy as u16
  }
}