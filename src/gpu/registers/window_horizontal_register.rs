use serde::{Deserialize, Serialize};

use crate::gpu::SCREEN_WIDTH;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct WindowHorizontalRegister {
  pub x1: u16,
  pub x2: u16,
  pub val: u16
}

impl WindowHorizontalRegister {
  pub fn new() -> Self {
    Self {
      x1: 0,
      x2: 0,
      val: 0
    }
  }

  pub fn write(&mut self, value: u16) {
    self.val = value;

    let mut x2 = value & 0xff;
    let x1 = value >> 8;

    if x1 > x2 || x2 > SCREEN_WIDTH {
      x2 = SCREEN_WIDTH;
    }

    self.x1 = x1;
    self.x2 = x2;
  }
}