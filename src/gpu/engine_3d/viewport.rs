use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Viewport {
  pub x1: u8,
  pub x2: u8,
  pub y1: u8,
  pub y2: u8
}

impl Viewport {
  pub fn new() -> Self {
    Self {
      x1: 0,
      x2: 0,
      y1: 0,
      y2: 0
    }
  }

  pub fn write(&mut self, value: u32) {
    self.x1 = value as u8;
    self.y1 = (value >> 8) as u8;
    self.x2 = (value >> 16) as u8;
    self.y2 = (value >> 24) as u8;
  }

  pub fn width(&self) -> i32 {
    (self.x2 - self.x1) as i32
  }

  pub fn height(&self) -> i32 {
    (self.y2 - self.y1) as i32
  }
}