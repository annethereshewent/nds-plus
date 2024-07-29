use std::cmp;

use crate::gpu::{color::Color, engine_3d::texture_params::TextureParams, vram::VRam, SCREEN_WIDTH};

use super::{polygon::Polygon, texcoord::Texcoord, texture_params::TextureFormat, vertex::Vertex, Engine3d, Pixel3d};

#[derive(Debug)]
pub struct TextureDeltas {
  pub dudx: f32,
  pub dudy: f32,
  pub dvdx: f32,
  pub dvdy: f32
}

impl TextureDeltas {
  pub fn new(dudx: f32, dudy: f32, dvdx: f32, dvdy: f32) -> Self {
    Self {
      dudx,
      dudy,
      dvdx,
      dvdy
    }
  }

  pub fn get_texture_deltas(v: &[Vertex], cross_product: i32) -> Self {
    let dudx_cp = Engine3d::cross_product(
      v[0].texcoord.u as i32, v[0].screen_y as i32,
      v[1].texcoord.u as i32, v[1].screen_y as i32,
      v[2].texcoord.u as i32, v[2].screen_y as i32
    );

    let dudy_cp = Engine3d::cross_product(
      v[0].screen_x as i32, v[0].texcoord.u as i32,
      v[1].screen_x as i32, v[1].texcoord.u as i32,
      v[2].screen_x as i32, v[2].texcoord.u as i32
    );

    let dvdx_cp = Engine3d::cross_product(
      v[0].texcoord.v as i32, v[0].screen_y as i32,
      v[1].texcoord.v as i32, v[1].screen_y as i32,
      v[2].texcoord.v as i32, v[2].screen_y as i32
    );

    let dvdy_cp = Engine3d::cross_product(
      v[0].screen_x as i32, v[0].texcoord.v as i32,
      v[1].screen_x as i32, v[1].texcoord.v as i32,
      v[2].screen_x as i32, v[2].texcoord.v as i32,
    );

    let dudx = dudx_cp as f32 / cross_product as f32;
    let dudy = dudy_cp as f32 / cross_product as f32;

    let dvdx = dvdx_cp as f32 / cross_product as f32;
    let dvdy = dvdy_cp as f32 / cross_product as f32;

    Self::new(dudx, dudy, dvdx, dvdy)
  }
}

impl Engine3d {
  pub fn cross_product(ax: i32, ay: i32, bx: i32, by: i32, cx: i32, cy: i32) -> i32 {
    (bx - ax) * (cy - ay) - (by - ay) * (cx - ax)
  }
  pub fn start_rendering(&mut self, vram: &VRam) {
    if self.polygons_ready {


      for polygon in self.polygon_buffer.drain(..) {
        // need to find the boundaries of the polygon and draw triangles
        let vertices = &mut self.vertices_buffer[polygon.start..polygon.end];

        if vertices.len() == 3 {
          Self::rasterize_triangle(&polygon, vertices, vram, &mut self.frame_buffer);
        } else {
          // break up into multiple triangles and then render the triangles that way lmao
          let mut i = 0;
          while i + 3 < vertices.len() {
            let mut cloned = [Vertex::new(); 3];

            cloned.clone_from_slice(&vertices[i..i + 3]);

            Self::rasterize_triangle(&polygon, &mut cloned, vram, &mut self.frame_buffer);

            i += 1;
          }
        }
      }


      self.vertices_buffer.clear();
      self.polygons_ready = false;
      self.gxstat.geometry_engine_busy = false;
    }
  }

  fn rasterize_triangle(polygon: &Polygon, vertices: &mut [Vertex], vram: &VRam, frame_buffer: &mut [Pixel3d]) {
    vertices.sort_by(|a, b| a.screen_y.cmp(&b.screen_y));

    let cross_product = Self::cross_product(
    vertices[0].screen_x as i32,
    vertices[0].screen_y as i32,
    vertices[1].screen_x as i32,
    vertices[1].screen_y as i32,
    vertices[2].screen_x as i32,
    vertices[2].screen_y as i32
    );

    if cross_product == 0 {
      println!("found a 0 cross product");
      return;
    }

    let texture_d = TextureDeltas::get_texture_deltas(vertices, cross_product);

    let min_y = cmp::min(vertices[0].screen_y, cmp::min(vertices[1].screen_y, vertices[2].screen_y));
    let max_y = cmp::max(vertices[0].screen_y, cmp::max(vertices[1].screen_y, vertices[2].screen_y));

    let min_x = cmp::min(vertices[0].screen_x, cmp::min(vertices[1].screen_x, vertices[2].screen_x));
    let max_x = cmp::max(vertices[0].screen_x, cmp::max(vertices[1].screen_x, vertices[2].screen_x));

    let mut u_base = vertices[0].texcoord.u as f32;
    let mut v_base = vertices[0].texcoord.v as f32;

    u_base -= texture_d.dudx * vertices[0].screen_x as f32 + texture_d.dudy * vertices[0].screen_y as f32;
    v_base -= texture_d.dvdx * vertices[0].screen_x as f32 + texture_d.dvdy * vertices[0].screen_y as f32;

    let p01_slope = if vertices[0].screen_y != vertices[1].screen_y {
      let slope = (vertices[1].screen_x as i32 - vertices[0].screen_x as i32) as f32 / (vertices[1].screen_y as i32 - vertices[0].screen_y as i32) as f32;
      Some(slope)
    } else {
      None
    };

    let p12_slope = if vertices[1].screen_y != vertices[2].screen_y {
      let slope = (vertices[2].screen_x as i32 - vertices[1].screen_x as i32) as f32 / (vertices[2].screen_y as i32 - vertices[1].screen_y as i32) as f32;
      Some(slope)
    } else {
      None
    };

    let p02_slope = if vertices[0].screen_y != vertices[2].screen_y {
      let slope = (vertices[2].screen_x as i32 - vertices[0].screen_x as i32) as f32 / (vertices[2].screen_y as i32 - vertices[0].screen_y as i32) as f32;
      Some(slope)
    } else {
      None
    };

    let texture_coordinates: Vec<[i16; 2]> = vertices.iter().map(|vertex| [vertex.texcoord.u, vertex.texcoord.v]).collect();

    println!("{:?}", texture_coordinates);

    let mut y = min_y;
    let mut x = min_x;

    while y < max_y {
      x = min_x;
      while x < max_x {
        let (boundary1, boundary2) = Self::get_triangle_boundaries(vertices, p01_slope, p02_slope, p12_slope, x as i32, y as i32);

        if (boundary1..boundary2).contains(&(x as i32)) {
          // render the pixel!
          // let pixel = &mut frame_buffer[(x + y * SCREEN_WIDTH as u32) as usize];

          // pixel.color = Some(vertices[0].color);

          let mut curr_u = (texture_d.dudx * x as f32 + texture_d.dudy * y as f32 + u_base) as u32;
          let mut curr_v = (texture_d.dvdx * x as f32 + texture_d.dudy * y as f32 + v_base) as u32;

          curr_u = curr_u.clamp(0, polygon.tex_params.texture_s_size());
          curr_v = curr_v.clamp(0, polygon.tex_params.texture_t_size());

          let texel = curr_u + curr_v * polygon.tex_params.texture_s_size();
          let vram_offset = polygon.tex_params.vram_offset();

          let pixel = &mut frame_buffer[(x + y * SCREEN_WIDTH as u32) as usize];

          match polygon.tex_params.texture_format() {
            TextureFormat::None => (),
            TextureFormat::A315Transluscent => {

            }
            TextureFormat::A513Transluscent => {
              let address = vram_offset + texel;

              let byte = vram.read_texture(address);

              let palette_index = byte & 0x3;

              let alpha = (byte >> 3) & 0x1f;

              let palette_base = polygon.palette_base;

              let address = palette_base as u32 + 2 * palette_index as u32;

              let color_raw = vram.read_texture_palette(address) as u16 | (vram.read_texture_palette(address + 1) as u16) << 8;

              pixel.color = if palette_index == 0 && polygon.tex_params.contains(TextureParams::COLOR0_TRANSPARENT) {
                None
              } else {
                Some(Color::from(color_raw))
              };

              pixel.alpha = alpha;
            }
            TextureFormat::Color16 => {

            }
            TextureFormat::Color256 => {

            }
            TextureFormat::Color4x4 => {

            }
            TextureFormat::Color4 => {

            }
            TextureFormat::Direct => {

            }
          }
        }

        x += 1;
      }
      y += 1;
    }
  }

  fn get_triangle_boundaries(vertices: &[Vertex], p01_slope: Option<f32>, p02_slope: Option<f32>, p12_slope: Option<f32>, x: i32, y: i32) -> (i32, i32) {
    let mut boundary2 = 0;

    // three cases to consider: p02 is always horizontal because vertices are sorted
    // by y coordinate, so either p01 slope is horizontal, p12 is, or neither are.
    if p01_slope.is_none() {
      let p12_slope = p12_slope.unwrap();

      let rel_y = y - vertices[1].screen_y as i32;

      boundary2 = ((p12_slope * rel_y as f32) + vertices[1].screen_x as f32) as i32;

    } else if p12_slope.is_none() {
      let p01_slope = p01_slope.unwrap();

      let rel_y = y as i32 - vertices[0].screen_y as i32;

      boundary2 = ((p01_slope * rel_y as f32) + vertices[0].screen_x as f32) as i32;
    } else {
      // neither slope is horizontal, determine which slope to use based on y coordinate.
      // if y coordinate is less than vertex 1's y coordinate, then use p01 slope
      // otherwise, boundary must be in p12 slope
      if y < vertices[1].screen_y as i32 {
        let p01_slope = p01_slope.unwrap();

        let rel_y = y - vertices[0].screen_y as i32;

        boundary2 = ((p01_slope * rel_y as f32) + vertices[0].screen_x as f32) as i32;
      } else {
        let p12_slope = p12_slope.unwrap();

        let rel_y = y - vertices[1].screen_y as i32;

        boundary2 = ((p12_slope * rel_y as f32) + vertices[1].screen_x as f32) as i32;
      }
    }

    let p02_slope = p02_slope.unwrap();

    let rel_y = y - vertices[0].screen_y as i32;

    let boundary1 = ((p02_slope * rel_y as f32) + vertices[0].screen_x as f32) as i32;

    if boundary2 > boundary1 {
      (boundary1, boundary2)
    } else {
      (boundary2, boundary1)
    }
  }
}