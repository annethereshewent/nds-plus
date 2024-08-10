use std::{cmp, collections::HashSet};

use crate::gpu::{color::Color, engine_3d::texture_params::TextureParams, registers::display_3d_control_register::Display3dControlRegister, vram::VRam, GPU, SCREEN_HEIGHT, SCREEN_WIDTH};

use super::{polygon::Polygon, polygon_attributes::{PolygonAttributes, PolygonMode}, texture_params::TextureFormat, vertex::Vertex, Engine3d, Pixel3d};


pub struct Deltas {
  pub dw: f32,
  pub dz: f32
}

impl Deltas {
  pub fn new(dz: f32, dw: f32) -> Self {
    Self {
      dw,
      dz
    }
  }

  pub fn get_deltas(start: Option<Vertex>, end: Option<Vertex>) -> Self {
    let start = start.unwrap();
    let end = end.unwrap();

    let num_steps = end.screen_y as f32 - start.screen_y as f32;

    let dw = (end.normalized_w as f32 - start.normalized_w as f32) / num_steps as f32;
    let dz = (end.z_depth as f32 - start.z_depth as f32) / num_steps as f32;

    Self::new(dz, dw)
  }
}


#[derive(Debug)]
pub struct Slope {
  current: usize,
  start: f32,
  num_steps: f32,
  w_start: f32,
  w_end: f32,
  diff: f32
}

impl Slope {
  pub fn new(start: f32, w_start: f32, w_end: f32, diff: f32, num_steps: f32) -> Self {
    Self {
      start,
      current: 0,
      w_start,
      w_end,
      diff,
      num_steps
    }
  }

  pub fn next(&mut self) -> f32 {
    let current = self.current as f32;
    let factor = (current * self.w_start) / (((self.num_steps - current) * self.w_end) + (current * self.w_start));
    self.current += 1;
    self.start + factor * self.diff
  }

  pub fn get_texture_slope(start: Option<Vertex>, end: Option<Vertex>, is_u: bool) -> Self {
    let start = start.unwrap();
    let end = end.unwrap();

    let (start_fp, end_fp) = if is_u {
      (start.texcoord.u as f32, end.texcoord.u as f32)
    } else {
      (start.texcoord.v as f32, end.texcoord.v as f32)
    };

    Slope::new(
      start_fp,
      start.normalized_w as f32,
      end.normalized_w as f32,
      end_fp - start_fp,
      (end.screen_y - start.screen_y) as f32
    )
  }
}

#[derive(Debug)]
pub struct RgbSlopes {
  r_slope: Slope,
  g_slope: Slope,
  b_slope: Slope
}

impl RgbSlopes {
  pub fn get_slopes(start: Option<Vertex>, end: Option<Vertex>) -> RgbSlopes {
    let start = start.unwrap();
    let end = end.unwrap();


    let r_slope = Self::new_slope(start, end, start.color.r as f32, end.color.r as f32);
    let g_slope = Self::new_slope(start, end, start.color.g as f32, end.color.g as f32);
    let b_slope = Self::new_slope(start, end, start.color.b as f32, end.color.b as f32);

    Self {
      r_slope,
      g_slope,
      b_slope
    }
  }
  /*
    left_rgb,
        end_rgb,
        w_start,
        w_end,
        (boundary2 - boundary1) as f32
   */
  pub fn new(start: Color, end: Color, w_start: f32, w_end: f32, num_steps: f32) -> Self {
    let r_slope = Slope::new(
      start.r as f32,
      w_start,
      w_end,
      (end.r as i32 - start.r as i32) as f32,
      num_steps
    );

    let g_slope = Slope::new(
      start.g as f32,
      w_start,
      w_end,
      (end.g as i32 - start.g as i32) as f32,
      num_steps
    );

    let b_slope = Slope::new(
      start.b as f32,
      w_start,
      w_end,
      (end.b as i32 - start.b as i32) as f32,
      num_steps
    );

    Self {
      r_slope,
      g_slope,
      b_slope
    }
  }

  pub fn new_slope(start: Vertex, end: Vertex, start_fp: f32, end_fp: f32) -> Slope {
    Slope::new(
      start_fp,
      start.normalized_w as f32,
      end.normalized_w as f32,
      end_fp - start_fp,
      (end.screen_y - start.screen_y) as f32
    )
  }

  pub fn next_color(&mut self) -> Color {
    let r = self.r_slope.next() as u8;
    let g = self.g_slope.next() as u8;
    let b = self.b_slope.next() as u8;

    Color {
      r,
      g,
      b,
      alpha: None
    }
  }
}


impl Engine3d {
  pub fn cross_product(ax: i32, ay: i32, bx: i32, by: i32, cx: i32, cy: i32) -> i32 {
    (bx - ax) * (cy - ay) - (by - ay) * (cx - ax)
  }

  pub fn start_rendering(&mut self, vram: &VRam) {
    if self.polygons_ready {
      if self.clear_color.alpha != 0 {
        for pixel in &mut self.frame_buffer {
          pixel.color = Some(Color {
            r: self.clear_color.r,
            g: self.clear_color.g,
            b: self.clear_color.b,
            alpha: Some(self.clear_color.alpha)
          });
          pixel.depth = self.clear_depth as u32;
        }
      } else {
        self.clear_frame_buffer();
      }

      for polygon in self.polygon_buffer.drain(..) {
        let vertices = &mut self.vertices_buffer[polygon.start..polygon.end];

        if vertices.len() == 3 {
          Self::rasterize_triangle(&polygon, vertices, vram, &mut self.frame_buffer, &self.toon_table, &self.disp3dcnt, self.debug_on, &mut self.found);
        } else {
          // break up into multiple triangles and then render the triangles
          let mut i = 0;
          vertices.sort_by(|a, b| {
            if a.screen_y != b.screen_y {
              a.screen_y.cmp(&b.screen_y)
            } else {
              a.screen_x.cmp(&b.screen_x)
            }
          });
          while i + 2 < vertices.len() {
            let mut cloned = [Vertex::new(); 3];

            cloned.clone_from_slice(&vertices[i..i + 3]);

            Self::rasterize_triangle(&polygon, &mut cloned, vram, &mut self.frame_buffer, &self.toon_table, &self.disp3dcnt, self.debug_on, &mut self.found);

            i += 1;
          }
        }
      }

      self.vertices_buffer.clear();
      self.polygons_ready = false;
      self.gxstat.geometry_engine_busy = false;
    }
  }

  fn get_palette_color(polygon: &Polygon, palette_base: u32, palette_index: u32, vram: &VRam, alpha: Option<u8>) -> (Option<Color>, Option<u8>) {
    let address = palette_base + 2 * palette_index;

    let color_raw = vram.read_texture_palette(address) as u16 | (vram.read_texture_palette(address + 1) as u16) << 8;

    if palette_index == 0 && polygon.tex_params.contains(TextureParams::COLOR0_TRANSPARENT) && alpha.is_none() {
      (Some(Color::from(color_raw).to_rgb6()), Some(0))
    } else {
      (Some(Color::from(color_raw).to_rgb6()), alpha)
    }
  }

  fn rasterize_triangle(
    polygon: &Polygon,
    vertices: &mut [Vertex],
    vram: &VRam,
    frame_buffer: &mut [Pixel3d],
    toon_table: &[Color],
    disp3dcnt: &Display3dControlRegister,
    debug_on: bool,
    found: &mut HashSet<String>)
  {
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
      return;
    }

    let p02_is_left = cross_product > 0;

    let min_y = cmp::min(vertices[0].screen_y, cmp::min(vertices[1].screen_y, vertices[2].screen_y));
    let max_y = cmp::max(vertices[0].screen_y, cmp::max(vertices[1].screen_y, vertices[2].screen_y));

    let min_x = cmp::min(vertices[0].screen_x, cmp::min(vertices[1].screen_x, vertices[2].screen_x));
    let max_x = cmp::max(vertices[0].screen_x, cmp::max(vertices[1].screen_x, vertices[2].screen_x));

    if max_x >= SCREEN_WIDTH as u32 || min_x >= SCREEN_WIDTH as u32 {
      return;
    }

    if max_y >= SCREEN_HEIGHT as u32 || min_y >= SCREEN_HEIGHT as u32 {
      return;
    }

    if (max_x - min_x) >= SCREEN_WIDTH as u32 {
      return;
    }

    if (max_y - min_y) >= SCREEN_HEIGHT as u32 {
      return;
    }

    let mut left_start: Option<Vertex> = None;
    let mut left_end: Option<Vertex> = None;

    let mut right_end: Option<Vertex> = None;
    let mut right_start: Option<Vertex> = None;


    let p01_slope = if vertices[0].screen_y != vertices[1].screen_y {
      let slope = (vertices[1].screen_x as i32 - vertices[0].screen_x as i32) as f32 / (vertices[1].screen_y as i32 - vertices[0].screen_y as i32) as f32;
      if p02_is_left {
        right_start = Some(vertices[0]);
        right_end =  Some(vertices[1]);
      } else {
        left_start = Some(vertices[0]);
        left_end = Some(vertices[1]);
      }
      Some(slope)
    } else {
      None
    };

    let p12_slope = if vertices[1].screen_y != vertices[2].screen_y {
      let slope = (vertices[2].screen_x as i32 - vertices[1].screen_x as i32) as f32 / (vertices[2].screen_y as i32 - vertices[1].screen_y as i32) as f32;

      if p02_is_left {
        right_start = Some(vertices[1]);
        right_end =  Some(vertices[2]);
      } else {
        left_start = Some(vertices[1]);
        left_end = Some(vertices[2]);
      }
      Some(slope)
    } else {
      None
    };

    let p02_slope = if vertices[0].screen_y != vertices[2].screen_y {
      let slope = (vertices[2].screen_x as i32 - vertices[0].screen_x as i32) as f32 / (vertices[2].screen_y as i32 - vertices[0].screen_y as i32) as f32;

      if p02_is_left {
        left_start = Some(vertices[0]);
        left_end =  Some(vertices[2]);
      } else {
        right_start = Some(vertices[0]);
        right_end = Some(vertices[2]);
      }

      Some(slope)
    } else {
      None
    };

    let mut left_vertical_u = Slope::get_texture_slope(left_start, left_end, true);
    let mut right_vertical_u = Slope::get_texture_slope(right_start, right_end, true);

    let mut left_vertical_v = Slope::get_texture_slope(left_start, left_end, false);
    let mut right_vertical_v = Slope::get_texture_slope(right_start, right_end, false);

    let mut left_vertical_rgb = RgbSlopes::get_slopes(left_start, left_end);
    let mut right_vertical_rgb = RgbSlopes::get_slopes(right_start, right_end);

    let left_vertical_delta = Deltas::get_deltas(left_start, left_end);
    let right_vertical_delta = Deltas::get_deltas(right_start, right_end);

    let mut y = min_y;
    let mut x = min_x;

    let mut w_start = left_start.unwrap().normalized_w as f32;
    let mut w_end = right_start.unwrap().normalized_w as f32;

    let mut z_start = left_start.unwrap().z_depth as f32;
    let mut z_end = right_start.unwrap().z_depth as f32;

    while y < max_y {
      let left_u = left_vertical_u.next();
      let right_u = right_vertical_u.next();

      let left_v = left_vertical_v.next();
      let right_v = right_vertical_v.next();

      let left_rgb = left_vertical_rgb.next_color();
      let right_rgb = right_vertical_rgb.next_color();

      let (boundary1, boundary2) = Self::get_triangle_boundaries(vertices, p01_slope, p02_slope, p12_slope, y as i32);

      x = boundary1 as u32;

      w_start += left_vertical_delta.dw;
      w_end += right_vertical_delta.dw;

      z_start += left_vertical_delta.dz;
      z_end += right_vertical_delta.dz;

      let mut u_d = Slope::new(
        left_u,
        w_start as f32,
        w_end as f32,
        right_u - left_u,
        (boundary2 - boundary1) as f32
      );

      let mut v_d = Slope::new(
        left_v,
        w_start as f32,
        w_end as f32,
        right_v - left_v,
        (boundary2 - boundary1) as f32
      );

      let mut rgb_d = RgbSlopes::new(
        left_rgb,
        right_rgb,
        w_start,
        w_end,
        (boundary2 - boundary1) as f32
      );

      let dzdx = (z_end - z_start) / (boundary2 as f32 - boundary1 as f32);

      let mut z = z_start;

      while x < boundary2 as u32 {
        let curr_u = u_d.next() as u32 >> 4;
        let curr_v = v_d.next() as u32 >> 4;

        z += dzdx;

        let mut vertex_color = rgb_d.next_color();

        vertex_color.alpha = Some(polygon.attributes.alpha());

        // render the pixel!
        let pixel = &mut frame_buffer[(x + y * SCREEN_WIDTH as u32) as usize];

        // println!("s,t: {curr_u},{curr_v} x,y: {x},{y}");

        let mut color: Option<Color> = None;

        let (texel_color, alpha) = Self::get_texel_color(polygon, curr_u, curr_v, vram, debug_on, found);
        if let Some(texel_color) = texel_color {
          color = if alpha.is_some() && alpha.unwrap() == 0 {
            None
          } else {
            // check to see if color is blended
            match polygon.attributes.polygon_mode() {
              PolygonMode::Decal => {
                todo!("decal mode not implemented");
              }
              PolygonMode::Modulation => {
                Self::modulation_blend(texel_color, vertex_color, alpha, false)
              }
              PolygonMode::Shadow => {
                todo!("shadow mode not implemented");
              }
              PolygonMode::Toon => {
                if disp3dcnt.contains(Display3dControlRegister::POLYGON_ATTR_SHADING) {
                  Self::modulation_blend(texel_color, vertex_color, alpha, true)
                } else {
                  let mut toon_color = toon_table[((vertex_color.r >> 1) & 0x1f) as usize];

                  toon_color.alpha = vertex_color.alpha;

                  toon_color.to_rgb6();

                  Self::modulation_blend(texel_color, toon_color, alpha, false)
                }
              }
            }
          }
        } else {
          color = Some(vertex_color);
        }

        if let Some(mut color) = color {
          if disp3dcnt.contains(Display3dControlRegister::ALPHA_BLENDING_ENABLE) && pixel.color.is_some() && pixel.color.unwrap().alpha.is_some() && color.alpha.is_some() {
            let fb_alpha = pixel.color.unwrap().alpha.unwrap();
            let polygon_alpha = color.alpha.unwrap();

            if fb_alpha != 0 {
              let pixel_color = pixel.color.unwrap().to_rgb6();
              let mut color = Self::blend_colors3d(pixel_color, color, 0x1f - polygon_alpha as u16, (polygon_alpha + 1) as u16);

              color.alpha = Some(cmp::max(fb_alpha, polygon_alpha));

              color.to_rgb5();

              pixel.color = Some(color);
            } else {
              color.to_rgb5();
              pixel.color = Some(color);
            }

            if polygon.attributes.contains(PolygonAttributes::UPDATE_DEPTH_FOR_TRANSLUSCENT) {
              pixel.depth = z as u32;
            }

          } else if Self::check_polygon_depth(polygon, pixel.depth, z as u32) {
            color.to_rgb5();
            pixel.color = Some(color);
            pixel.depth = z as u32;
          }
        }
        x += 1;
      }
      y += 1;
    }
  }

  fn check_polygon_depth(polygon: &Polygon, current_depth: u32, new_depth: u32) -> bool {
    if polygon.attributes.contains(PolygonAttributes::DRAW_PIXELS_WITH_DEPTH) {
      new_depth >= current_depth - 0x200 && new_depth <= current_depth + 0x200
    } else {
      new_depth < current_depth
    }
  }

  pub fn blend_colors3d(color: Color, color2: Color, eva: u16, evb: u16) -> Color {
    let r = ((color.r as u16 * eva + color2.r as u16 * evb) >> 5) as u8;
    let g = ((color.g as u16 * eva + color2.g as u16 * evb) >> 5) as u8;
    let b = ((color.b as u16 * eva + color2.b as u16 * evb) >> 5) as u8;

    Color {
      r,
      g,
      b,
      alpha: None
    }
  }

  fn modulation_blend(texel: Color, pixel: Color, alpha: Option<u8>, toon_highlight: bool) -> Option<Color> {
    // ((val1 + 1) * (val2 + 1) - 1) / 64;
    let modulation_fn = |component1, component2| ((component1 + 1) * (component2 + 1) - 1) / 64;

    let mut r = modulation_fn(texel.r as u16, pixel.r as u16) as u8;
    let mut g = modulation_fn(texel.g as u16, pixel.g as u16) as u8;
    let mut b = modulation_fn(texel.b as u16, pixel.b as u16) as u8;

    let new_alpha = if pixel.alpha.is_some() && alpha.is_some() {
      Some(modulation_fn(pixel.alpha.unwrap() as u16, alpha.unwrap() as u16) as u8)
    } else {
      alpha
    };

    if toon_highlight {
      r = cmp::max(r + pixel.r, 0x3f);
      g = cmp::max(g + pixel.g, 0x3f);
      b = cmp::max(b + pixel.b, 0x3f);
    }

    Some(Color {
      r,
      g,
      b,
      alpha: new_alpha
    })
  }

  fn check_if_texture_repeated(val: u32, repeat: bool, flip: bool, mask: u32, shift: u32) -> u32 {
    let mut return_val = val;
    if repeat {
      return_val &= mask;
      if flip && (val >> shift) % 2 == 1 {
        return_val ^= mask;
      }
    }

    return_val
  }

  fn get_texel_color(polygon: &Polygon, curr_u: u32, curr_v: u32, vram: &VRam, debug_on: bool, found: &mut HashSet<String>) -> (Option<Color>, Option<u8>) {
    let mut u = curr_u;

    u = Self::check_if_texture_repeated(
      u,
      polygon.tex_params.contains(TextureParams::REPEAT_S),
      polygon.tex_params.contains(TextureParams::FLIP_S),
      polygon.tex_params.texture_s_size() -1,
      polygon.tex_params.size_s_shift()
    );

    u = u.clamp(0, polygon.tex_params.texture_s_size() - 1);

    let mut v = curr_v;

    v = Self::check_if_texture_repeated(
      v,
      polygon.tex_params.contains(TextureParams::REPEAT_T),
      polygon.tex_params.contains(TextureParams::FLIP_T),
      polygon.tex_params.texture_t_size() -1,
      polygon.tex_params.size_t_shift()
    );

    v = v.clamp(0, polygon.tex_params.texture_t_size() - 1);

    // println!("got {u},{v}");


    let texel = u + v * polygon.tex_params.texture_s_size();
    let vram_offset = polygon.tex_params.vram_offset();

    let address = vram_offset + texel;

    let palette_base = polygon.palette_base;

    // println!("vram offset = {:x} palette base = {:x}", vram_offset, palette_base);

    match polygon.tex_params.texture_format() {
      TextureFormat::None => {
        (None, None)
      },
      TextureFormat::A315Transluscent => {
        let byte = vram.read_texture(address);

        let palette_index = byte & 0x1f;
        let alpha = (byte >> 5) & 0x7;

        Self::get_palette_color(polygon, palette_base as u32, palette_index as u32, vram, Some(alpha * 4 + alpha / 2))
      }
      TextureFormat::A513Transluscent => {
        let byte = vram.read_texture(address);

        let palette_index = byte & 0x7;

        let alpha = (byte >> 3) & 0x1f;

        Self::get_palette_color(polygon, palette_base as u32, palette_index as u32, vram, Some(alpha))
      }
      TextureFormat::Color16 => {
        let real_address = vram_offset + texel / 2;

        let byte = vram.read_texture(real_address);

        let palette_index = if texel & 0b1 == 0 {
          byte & 0xf
        } else {
          (byte >> 4) & 0xf
        };

        Self::get_palette_color(polygon, palette_base as u32, palette_index as u32, vram, None)
      }
      TextureFormat::Color256 => {
        let palette_index = vram.read_texture(address);

        Self::get_palette_color(polygon, palette_base as u32, palette_index as u32, vram, None)
      }
      TextureFormat::Color4x4 => {
        let blocks_per_row = polygon.tex_params.texture_s_size() / 4;

        let block_address = (u / 4) + blocks_per_row * (v / 4);

        let base_address = vram_offset + 4 * block_address;

        let mut texel_value = vram.read_texture(base_address + (v & 0x3));

        texel_value = match u & 0x3 {
          0 => texel_value & 0x3,
          1 => (texel_value >> 2) & 0x3,
          2 => (texel_value >> 4) & 0x3,
          3 => (texel_value >> 6) & 0x3,
          _ => unreachable!()
        };

        let slot1_address = 128 * 0x400 + (base_address & 0x1_ffff) / 2 + if base_address >= 128 * 0x400 {
          0x1000
        } else {
          0
        };

        let extra_palette_info = vram.read_texture(slot1_address) as u16 | (vram.read_texture(slot1_address + 1) as u16) << 8;

        let palette_offset = palette_base as u32 + ((extra_palette_info & 0x3fff) * 4) as u32;

        let mode = (extra_palette_info >> 14) & 0x3;

        let get_color = |num: u32|
          vram.read_texture_palette(palette_offset + 2 * num) as u16 | (vram.read_texture_palette(palette_offset + 2 * num + 1) as u16) << 8;

        match (texel_value, mode) {
          (0, _) => {
            // color 0
            (Some(Color::from(get_color(0)).to_rgb6()), None)
          }
          (1, _) => {
            // color 1
            (Some(Color::from(get_color(1)).to_rgb6()), None)
          },
          (2, 0) | (2, 2) => {
            // color 2
            (Some(Color::from(get_color(2)).to_rgb6()), None)
          }
          (2, 1) => {
            // (color0 + color1) / 2
            let color0 = Color::from(get_color(0));
            let color1 = Color::from(get_color(1));

            let mut blended_color = color0.blend_half(color1);

            (Some(blended_color.to_rgb6()), None)
          }
          (2, 3) => {
            // (color0 * 5 + color1 * 3) / 8

            let color0 = Color::from(get_color(0));
            let color1 = Color::from(get_color(1));

            let mut blended_color = color0.blend_texture(color1);

            (Some(blended_color.to_rgb6()), None)
          }
          (3, 0)| (3, 1) => {
            // transparent
            (Some(Color { r: 0, g: 0, b: 0, alpha: Some(0) }), Some(0))
          }
          (3, 2) => {
            // color 3
            (Some(Color::from(get_color(3)).to_rgb6()), None)
          }
          (3, 3) => {
            // (color0 * 3 + color1 * 5) / 8
            let color0 = Color::from(get_color(0));
            let color1 = Color::from(get_color(1));

            let mut blended_color = color1.blend_texture(color0);

            (Some(blended_color.to_rgb6()), None)
          }
          _ => panic!("invalid options given for texel value and mode: {texel_value} {mode}")
        }
      }
      TextureFormat::Color4 => {
        let mut palette_index = vram.read_texture(vram_offset + texel / 4);

        palette_index = match texel & 0x3 {
          0 => palette_index & 0x3,
          1 => (palette_index >> 2) & 0x3,
          2 => (palette_index >> 4) & 0x3,
          3 => (palette_index >> 6) & 0x3,
          _ => unreachable!()
        };

        let address = palette_base as u32 / 2 + palette_index as u32 * 2;
        let color_raw = vram.read_texture_palette(address) as u16 | (vram.read_texture_palette(address + 1) as u16) << 8;

        let alpha = if palette_index == 0 && polygon.tex_params.contains(TextureParams::COLOR0_TRANSPARENT) {
          Some(0)
        } else {
          None
        };

        (Some(Color::from(color_raw).to_rgb6()), alpha)
      }
      TextureFormat::Direct => {
        let address = vram_offset + 2 * texel;
        let color_raw = vram.read_texture(address) as u16 | (vram.read_texture(address + 1) as u16) << 8;

        let alpha = if color_raw & 0x8000 == 0 { 0 } else { 0x1f };

        (Some(Color::from(color_raw).to_rgb6()), Some(alpha))
      }
    }
  }

  fn get_triangle_boundaries(vertices: &[Vertex], p01_slope: Option<f32>, p02_slope: Option<f32>, p12_slope: Option<f32>, y: i32) -> (i32, i32) {
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