use serde::{Deserialize, Serialize};

use crate::gpu::color::Color;


#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Light {
  pub color: Color,
  pub x: i16,
  pub y: i16,
  pub z: i16,
  pub half_vector: [i32; 3]
}

impl Light {
  pub fn new() -> Self {
    Self {
      color: Color::new(),
      x: 0,
      y: 0,
      z: 0,
      half_vector: [0; 3]
    }
  }
}