use std::cmp;

use crate::gpu::vram::VRam;

use super::{polygon::Polygon, vertex::Vertex, Engine3d};

impl Engine3d {
  pub fn cross_product(a: Vertex, b: Vertex, c: Vertex) -> i32 {
    (b.screen_x as i32 - a.screen_x as i32) * (c.screen_y as i32 - a.screen_y as i32) - (b.screen_y as i32 - a.screen_y as i32) * (c.screen_x as i32 - a.screen_x as i32)
  }
  pub fn start_rendering(&mut self, vram: &VRam) {
    if self.polygons_ready {


      for polygon in self.polygon_buffer.drain(..) {
        // need to find the boundaries of the polygon and draw triangles
        let vertices = &mut self.vertices_buffer[polygon.start..polygon.end];

        if vertices.len() == 3 {
          Self::rasterize_triangle(&polygon, vertices, vram);
        } else {
          Self::rasterize_quad(&polygon, vertices, vram);
        }
      }


      self.vertices_buffer.clear();
      self.polygons_ready = false;
      self.gxstat.geometry_engine_busy = false;
    }
  }

  fn rasterize_triangle(polygon: &Polygon, vertices: &mut [Vertex], vram: &VRam) {
    vertices.sort_by(|a, b| a.screen_y.cmp(&b.screen_y));


    let cross_product = Self::cross_product(vertices[0], vertices[1], vertices[2]);

    if cross_product == 0 {
      return;
    }

    let min_y = cmp::min(vertices[0].screen_y, cmp::min(vertices[1].screen_y, vertices[2].screen_y));
    let max_y = cmp::max(vertices[0].screen_y, cmp::max(vertices[1].screen_y, vertices[2].screen_y));

    let min_x = cmp::min(vertices[0].screen_x, cmp::min(vertices[1].screen_x, vertices[2].screen_x));
    let max_x = cmp::max(vertices[0].screen_x, cmp::max(vertices[1].screen_x, vertices[2].screen_x));

    let coordinates: Vec<[u32; 2]> = vertices.iter().map(|vertex| [vertex.screen_x, vertex.screen_y]).collect();

    let p01_slope = if vertices[0].screen_y != vertices[1].screen_y {
      let slope = (vertices[1].screen_x - vertices[0].screen_x) as f32 / (vertices[1].screen_y - vertices[0].screen_y) as f32;
      Some(slope)
    } else {
      None
    };

    let p12_slope = if vertices[1].screen_y != vertices[2].screen_y {
      let slope = (vertices[2].screen_x - vertices[1].screen_x) as f32 / (vertices[2].screen_y - vertices[1].screen_y) as f32;
      Some(slope)
    } else {
      None
    };

    let p02_slope = if vertices[0].screen_y != vertices[2].screen_y {
      let slope = (vertices[2].screen_x - vertices[0].screen_x) as f32 / (vertices[2].screen_y - vertices[0].screen_y) as f32;
      Some(slope)
    } else {
      None
    };

    let mut y = min_y;
    let mut x = min_x;

    while y < max_y {
      x = min_x;
      while x < max_x {
        let (boundary1, boundary2) = Self::get_triangle_boundaries(vertices, p01_slope, p02_slope, p12_slope, x, y);

        if (boundary1..boundary2).contains(&x) {
          // render the pixel!

        }

        x += 1;
      }
      y += 1;
    }
  }

  fn get_triangle_boundaries(vertices: &[Vertex], p01_slope: Option<f32>, p02_slope: Option<f32>, p12_slope: Option<f32>, x: u32, y: u32) -> (u32, u32) {
    let mut boundary2 = 0;

    // three cases to consider: p02 is always horizontal because vertices are sorted
    // by y coordinate, so either p01 slope is horizontal, p12 is, or neither are.
    if p01_slope.is_none() {
      let p12_slope = p12_slope.unwrap();

      let rel_y = y - vertices[1].screen_y;

      boundary2 = ((p12_slope * rel_y as f32) + vertices[1].screen_x as f32) as u32;

    } else if p12_slope.is_none() {
      let p01_slope = p01_slope.unwrap();

      let rel_y = y - vertices[0].screen_y;


      boundary2 = ((p01_slope * rel_y as f32) + vertices[0].screen_x as f32) as u32;
    } else {
      // neither slope is horizontal, determine which slope to use based on y coordinate.
      // if y coordinate is less than vertex 1's y coordinate, then use p01 slope
      // otherwise, boundary must be in p12 slope
      if y < vertices[1].screen_y {
        let p01_slope = p01_slope.unwrap();

        let rel_y = y - vertices[0].screen_y;

        boundary2 = ((p01_slope * rel_y as f32) + vertices[0].screen_x as f32) as u32;
      }

    }

    let p02_slope = p02_slope.unwrap();

    let rel_y = y - vertices[0].screen_y;

    let boundary1 = ((p02_slope * rel_y as f32) + vertices[0].screen_x as f32) as u32;

    if boundary2 > boundary1 {
      (boundary1, boundary2)
    } else {
      (boundary2, boundary1)
    }
  }

  fn rasterize_quad(polygon: &Polygon, vertices: &mut [Vertex], vram: &VRam) {
    let mut first_vertices = [Vertex::new(); 4];

    first_vertices.clone_from_slice(&vertices[..]);

    Self::rasterize_triangle(polygon, &mut first_vertices[0..3], vram);
    Self::rasterize_triangle(polygon, &mut vertices[1..4], vram);
  }
}