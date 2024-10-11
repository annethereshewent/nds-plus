use serde::{Deserialize, Serialize};

use crate::gpu::color::Color;

enum BrightnessMode {
  Disabled = 0,
  Brighten = 1,
  Darken = 2
}

#[derive(Serialize, Deserialize)]
pub struct MasterBrightnessRegister {
  val: u16
}

impl MasterBrightnessRegister {
  pub fn new() -> Self {
    Self {
      val: 0
    }
  }

  pub fn read(&self) -> u16 {
    self.val
  }

  pub fn write(&mut self, val: u16) {
    self.val = val;
  }

  pub fn factor(&self) -> u16 {
    self.val & 0x1f
  }

  fn mode(&self) -> BrightnessMode {
    match self.val >> 14 & 0x3 {
      0 => BrightnessMode::Disabled,
      1 => BrightnessMode::Brighten,
      2 => BrightnessMode::Darken,
      _ => panic!("invalid mode given for master bright")
    }
  }

  pub fn apply_effect(&self, mut color: Color) -> Color {
    let factor = if self.factor() > 16 {
      16
    } else {
      self.factor()
    };
    match self.mode() {
      BrightnessMode::Disabled => color,
      BrightnessMode::Brighten => {
        color.r = (color.r as u16 + ((63 - color.r as u16) * factor) / 16) as u8;
        color.g = (color.g as u16 + ((63 - color.g as u16) * factor) / 16) as u8;
        color.b = (color.b as u16 + ((63 - color.b as u16) * factor) / 16) as u8;

        color
      }
      BrightnessMode::Darken => {
        color.r = (color.r as u16 - (color.r as u16 * factor) / 16) as u8;
        color.g = (color.g as u16 - (color.g as u16 * factor) / 16) as u8;
        color.b = (color.b as u16 - (color.b as u16 * factor) / 16) as u8;


        color
      }
    }
  }
}