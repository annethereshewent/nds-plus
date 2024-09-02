use super::{matrix::Matrix, vertex::Vertex, Engine3d};

pub struct BoxTest {
  pub x: i16,
  pub y: i16,
  pub z: i16,
  pub width: i16,
  pub height: i16,
  pub depth: i16
}

impl BoxTest {
  pub fn new() -> Self {
    Self {
      x: 0,
      y: 0,
      z: 0,
      width: 0,
      height: 0,
      depth: 0
    }
  }

  pub fn do_test(&mut self, clip_matrix: Matrix) -> bool {
    let mut vertices_arr: [Vertex; 8] = [Vertex::new(); 8];

    for x_factor in 0..2 {
      for y_factor in 0..2 {
        for z_factor in 0..2 {
          let coordinates = [
            self.x as i32 + self.width as i32 * x_factor as i32,
            self.y as i32 + self.height as i32 * y_factor as i32,
            self.z as i32 + self.depth as i32 * z_factor as i32,
            0x1000
          ];

          let transformed = clip_matrix.multiply_row(&coordinates, 12);

          let mut vertex = Vertex::new();

          vertex.transformed = transformed;

          vertices_arr[x_factor << 2 | y_factor << 1 | z_factor] = vertex;
        }
      }
    }

    for x in 0..2 {
      let mut vertices = [vertices_arr[x << 2], vertices_arr[x << 2 | 1], vertices_arr[x << 2 | 3], vertices_arr[x << 2 | 2]].to_vec();

      Engine3d::sutherland_hodgman_clipping(0, &mut vertices);

      if !vertices.is_empty() {
        return true;
      }
    }

    for y in 0..2 {
      let mut vertices = [vertices_arr[y << 1], vertices_arr[y << 1 | 1], vertices_arr[y << 1 | 5], vertices_arr[y << 1 | 4]].to_vec();

      Engine3d::sutherland_hodgman_clipping(1, &mut vertices);

      if !vertices.is_empty() {
        return true;
      }
    }

    for z in 0..2 {
      let mut vertices = [vertices_arr[z], vertices_arr[z | 2], vertices_arr[z | 6], vertices_arr[z | 4]].to_vec();

      Engine3d::sutherland_hodgman_clipping(2, &mut vertices);

      if !vertices.is_empty() {
        return true;
      }
    }

    false
  }
}