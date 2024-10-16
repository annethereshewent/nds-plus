use serde::{Deserialize, Serialize};

use crate::gpu::color::Color;

use super::texcoord::Texcoord;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Vertex {
  pub screen_x: u32,
  pub screen_y: u32,
  pub z_depth: u32,
  pub x: i16,
  pub y: i16,
  pub z: i16,
  pub transformed: [i32; 4],
  pub texcoord: Texcoord,
  pub color: Color,
  pub normalized_w: i16
}

impl Vertex {
  pub fn new() -> Self {
    Self {
      screen_x: 0,
      screen_y: 0,
      z_depth: 0,
      x: 0,
      y: 0,
      z: 0,
      transformed: [0; 4],
      texcoord: Texcoord::new(),
      color: Color::new(),
      normalized_w: 0
    }
  }
}