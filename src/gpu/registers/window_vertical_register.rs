use serde::{Deserialize, Serialize};

use crate::gpu::SCREEN_HEIGHT;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct WindowVerticalRegister {
  pub val: u16,
  pub y1: u16,
  pub y2: u16
}

impl WindowVerticalRegister {
  pub fn new() -> Self {
    Self {
      y1: 0,
      y2: 0,
      val: 0
    }
  }

  pub fn write(&mut self, value: u16) {
    self.val = value;

    let mut y2 = value & 0xff;
    let y1 = value >> 8;

    if y1 > y2 || y2 > SCREEN_HEIGHT {
      y2 = SCREEN_HEIGHT;
    }

    self.y1 = y1;
    self.y2 = y2;
  }
}