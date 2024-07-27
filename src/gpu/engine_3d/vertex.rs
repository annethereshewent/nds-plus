pub struct Vertex {
  screen_x: u32,
  screen_y: u32,
  z_depth: u32
}

impl Vertex {
  pub fn new() -> Self {
    Self {
      screen_x: 0,
      screen_y: 0,
      z_depth: 0
    }
  }
}