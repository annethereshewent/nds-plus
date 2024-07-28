use crate::gpu::color::Color;

use super::texcoord::Texcoord;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
  pub screen_x: u32,
  pub screen_y: u32,
  pub z_depth: u32,
  pub x: i16,
  pub y: i16,
  pub z: i16,
  pub texcoord: Texcoord,
  pub color: Color
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
      texcoord: Texcoord::new(),
      color: Color::new()
    }
  }
}