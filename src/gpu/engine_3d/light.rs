use crate::gpu::color::Color;


#[derive(Copy, Clone, Debug)]
pub struct Light {
  pub color: Color,
  pub x: i16,
  pub y: i16,
  pub z: i16
}

impl Light {
  pub fn new() -> Self {
    Self {
      color: Color::new(),
      x: 0,
      y: 0,
      z: 0
    }
  }
}