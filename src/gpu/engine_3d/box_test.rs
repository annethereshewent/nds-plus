use super::matrix::Matrix;

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

          let w = transformed[3];
          let mut inside = true;
          for i in 0..3 {
            if !(-w..=w).contains(&transformed[i]) {
               inside = false;
               break;
            }
          }

          if inside {
            return true;
          }
        }
      }
    }

    false
  }
}