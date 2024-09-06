use std::{cmp, collections::HashSet};

use crate::gpu::{
  color::Color,
  registers::display_3d_control_register::Display3dControlRegister,
  vram::VRam,
  SCREEN_HEIGHT,
  SCREEN_WIDTH
};

use super::{
  polygon::Polygon,
  polygon_attributes::{
    PolygonAttributes,
    PolygonMode
  },
  rendering_attributes::RenderingAttributes,
  texture_params::TextureFormat,
  vertex::Vertex,
  Engine3d,
  Pixel3d
};

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

  pub fn get_deltas(start: Vertex, end: Vertex) -> Self {
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

  pub fn get_texture_slope(start: Vertex, end: Vertex, is_u: bool) -> Self {
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
  pub fn get_slopes(start: Vertex, end: Vertex) -> RgbSlopes {
    let r_slope = Self::new_slope(start, end, start.color.r as f32, end.color.r as f32);
    let g_slope = Self::new_slope(start, end, start.color.g as f32, end.color.g as f32);
    let b_slope = Self::new_slope(start, end, start.color.b as f32, end.color.b as f32);

    Self {
      r_slope,
      g_slope,
      b_slope
    }
  }
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
  pub fn start_rendering(&mut self, vram: &VRam) {
    if self.polygons_ready {
      self.clear_attributes_buffer();
      if self.clear_color.alpha != 0 {
        for pixel in self.frame_buffer.iter_mut() {
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

      let mut render = |polygon, vertices| Self::render_polygon(
        &polygon,
        vertices,
        vram,
        &mut self.frame_buffer,
        &self.toon_table,
        &self.disp3dcnt,
        &mut self.fog_color,
        self.fog_offset,
        &self.fog_table,
        &mut self.attributes_buffer,
        self.debug_on,
        &mut self.found
      );


      if self.disp3dcnt.contains(Display3dControlRegister::ALPHA_BLENDING_ENABLE) {
        let (opaque, translucent): (Vec<Polygon>, Vec<Polygon>) =
          self.polygon_buffer.drain(..).partition(|polygon| polygon.attributes.alpha() == 0x1f);

        for polygon in opaque {
          let vertices = &self.vertices_buffer[polygon.start..polygon.end];
          render(polygon, vertices);
        }

        for polygon in translucent {
          let vertices = &self.vertices_buffer[polygon.start..polygon.end];
          render(polygon, vertices);
        }


      } else {
        for polygon in self.polygon_buffer.drain(..) {
          let vertices = &self.vertices_buffer[polygon.start..polygon.end];
          render(polygon, vertices);
        }
      }


      self.vertices_buffer.clear();
      self.polygons_ready = false;
      self.gxstat.geometry_engine_busy = false;
    }
  }

  fn get_palette_color(polygon: &Polygon, palette_base: u32, palette_index: u32, vram: &VRam, alpha: Option<u8>) -> Option<Color> {
    let address = palette_base + 2 * palette_index;

    let color_raw = vram.read_texture_palette::<u16>(address);

    let mut color = Color::from(color_raw).to_rgb6();

    if palette_index == 0 && polygon.tex_params.color0_transparent {
      color.alpha = Some(0);
    } else {
      color.alpha = alpha;
    }

    Some(color)
  }

  fn calculate_slope(start: Vertex, end: Vertex) -> f32 {
    if start.screen_y != end.screen_y {
      let slope = (end.screen_x as f32 - start.screen_x as f32) / (end.screen_y as f32 - start.screen_y as f32);

      slope
    } else {
      0.0
    }
  }

  fn render_polygon(
    polygon: &Polygon,
    vertices: &[Vertex],
    vram: &VRam,
    frame_buffer: &mut [Pixel3d],
    toon_table: &[Color],
    disp3dcnt: &Display3dControlRegister,
    fog_color: &mut Color,
    fog_offset: u16,
    fog_table: &[u8],
    attributes_buffer: &mut [RenderingAttributes],
    debug_on: bool,
    found: &mut HashSet<String>
  ) {
    if polygon.attributes.polygon_mode() == PolygonMode::Shadow {
      // TODO: implement shadow mode
      return;
    }

    let mut min_y = vertices[0].screen_y;
    let mut max_y = vertices[0].screen_y;
    let mut min_x = vertices[0].screen_x;
    let mut max_x = vertices[0].screen_x;

    let mut start_index = 0;
    let mut end_index = 0;

    for i in 1..vertices.len() {
      let current = &vertices[i];

      if current.screen_y < vertices[start_index].screen_y {
        start_index = i;
        min_y = current.screen_y;
      } else if current.screen_y == vertices[start_index].screen_y && current.screen_x < vertices[start_index].screen_x {
        start_index = i;
      }

      if current.screen_y > vertices[end_index].screen_y {
        end_index = i;
        max_y = current.screen_y;
      } else if current.screen_y == vertices[end_index].screen_y && current.screen_x >= vertices[end_index].screen_x {
        end_index = i;
      }

      if current.screen_x < min_x {
        min_x = current.screen_x;
      }
      if current.screen_x > max_x {
        max_x = current.screen_x;
      }
    }

    max_y = max_y.min(SCREEN_HEIGHT as u32 - 1);

    let mut left_start_index = start_index;
    let mut right_start_index = start_index;

    let next = |index| (index + 1) % vertices.len();
    let previous = |index| {
      if index == 0 {
        vertices.len() - 1
      } else {
        index - 1
      }
    };

    let next_left: Box<dyn Fn(usize) -> usize> = if polygon.is_front {
      Box::new(next)
    } else {
      Box::new(previous)
    };

    let next_right: Box<dyn Fn(usize) -> usize> = if !polygon.is_front {
      Box::new(next)
    } else {
      Box::new(previous)
    };

    let left_end_index = next_left(left_start_index);
    let right_end_index = next_right(right_start_index);

    let mut left_start = vertices[left_start_index];
    let mut left_end = vertices[left_end_index];

    let mut right_start = vertices[right_start_index];
    let mut right_end = vertices[right_end_index];

    left_start_index = left_end_index;
    right_start_index = right_end_index;

    let mut left_slope = Self::calculate_slope(left_start, left_end);
    let mut right_slope = Self::calculate_slope(right_start, right_end);

    let mut left_vertical_u = Slope::get_texture_slope(left_start, left_end, true);
    let mut right_vertical_u = Slope::get_texture_slope(right_start, right_end, true);

    let mut left_vertical_v = Slope::get_texture_slope(left_start, left_end, false);
    let mut right_vertical_v = Slope::get_texture_slope(right_start, right_end, false);

    let mut left_vertical_rgb = RgbSlopes::get_slopes(left_start, left_end);
    let mut right_vertical_rgb = RgbSlopes::get_slopes(right_start, right_end);

    let mut left_vertical_delta = Deltas::get_deltas(left_start, left_end);
    let mut right_vertical_delta = Deltas::get_deltas(right_start, right_end);

    let mut y = min_y;
    let mut x = min_x;

    let mut w_start = left_start.normalized_w as f32;
    let mut w_end = right_start.normalized_w as f32;

    let mut z_start = left_start.z_depth as f32;
    let mut z_end = right_start.z_depth as f32;

    let mut boundary1 = left_start.screen_x as f32;
    let mut boundary2 = right_start.screen_x as f32;

    while y < max_y {
      while y >= left_end.screen_y {
        // need to calculate a new left slope
        let left_end_index = next_left(left_start_index);

        left_start = left_end;
        left_end = vertices[left_end_index];

        left_start_index = left_end_index;

        if y < left_end.screen_y {
          left_slope = Self::calculate_slope(left_start, left_end);

          left_vertical_u = Slope::get_texture_slope(left_start, left_end, true);
          left_vertical_v = Slope::get_texture_slope(left_start, left_end, false);

          left_vertical_rgb = RgbSlopes::get_slopes(left_start, left_end);
          left_vertical_delta = Deltas::get_deltas(left_start, left_end);

          boundary1 = left_start.screen_x as f32;
        }
      }
      while y >= right_end.screen_y {
        // need to calculate a new right slope
        let right_end_index = next_right(right_start_index);

        right_start = right_end;
        right_end = vertices[right_end_index];

        right_start_index = right_end_index;

        if y < right_end.screen_y {
          right_slope = Self::calculate_slope(right_start, right_end);

          right_vertical_u = Slope::get_texture_slope(right_start, right_end, true);
          right_vertical_v = Slope::get_texture_slope(right_start, right_end, false);

          right_vertical_rgb = RgbSlopes::get_slopes(right_start, right_end);
          right_vertical_delta = Deltas::get_deltas(right_start, right_end);

          boundary2 = right_start.screen_x as f32;
        }
      }

      let left_u = left_vertical_u.next();
      let right_u = right_vertical_u.next();

      let left_v = left_vertical_v.next();
      let right_v = right_vertical_v.next();

      let left_rgb = left_vertical_rgb.next_color();
      let right_rgb = right_vertical_rgb.next_color();

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
        boundary2 - boundary1
      );

      let mut v_d = Slope::new(
        left_v,
        w_start,
        w_end,
        right_v - left_v,
        boundary2 - boundary1
      );

      let mut rgb_d = RgbSlopes::new(
        left_rgb,
        right_rgb,
        w_start,
        w_end,
        boundary2 - boundary1
      );

      let dzdx = (z_end - z_start) / (boundary2 - boundary1);

      let mut z = z_start;

      let boundary2_u32 = boundary2 as u32;

      while x < boundary2_u32 && x < SCREEN_WIDTH as u32 {
        let curr_u = u_d.next() as i32 >> 4;
        let curr_v = v_d.next() as i32 >> 4;

        let mut vertex_color = rgb_d.next_color();

        vertex_color.alpha = Some(polygon.attributes.alpha());

        // render the pixel!
        let pixel = &mut frame_buffer[(x + y * SCREEN_WIDTH as u32) as usize];
        let prev_attributes = attributes_buffer[(x + y * SCREEN_WIDTH as u32) as usize];

        let mut color: Option<Color> = None;

        if let Some(texel_color) = Self::get_texel_color(polygon, curr_u, curr_v, vram, debug_on, found) {
          color = if texel_color.alpha.is_some() && texel_color.alpha.unwrap() == 0 {
            None
          } else {
            // check to see if color is blended
            match polygon.attributes.polygon_mode() {
              PolygonMode::Decal => {
                todo!("decal mode not implemented");
              }
              PolygonMode::Modulation => {
                Self::modulation_blend(texel_color, vertex_color)
              }
              PolygonMode::Shadow => {
                todo!("shadow mode not implemented");
              }
              PolygonMode::Toon => {
                if disp3dcnt.contains(Display3dControlRegister::POLYGON_ATTR_SHADING) {
                  let shaded = Color {
                    r: vertex_color.r,
                    g: vertex_color.r,
                    b: vertex_color.r,
                    alpha: Some(vertex_color.r)
                  };
                  let blended = Self::modulation_blend(texel_color, shaded);

                  let mut toon_color = toon_table[((vertex_color.r >> 1) & 0x1f) as usize];

                  toon_color.alpha = vertex_color.alpha;

                  toon_color.to_rgb6();

                  Self::apply_polygon_shading(blended.unwrap(), toon_color)
                } else {
                  let mut toon_color = toon_table[((vertex_color.r >> 1) & 0x1f) as usize];

                  toon_color.alpha = vertex_color.alpha;

                  toon_color.to_rgb6();

                  Self::modulation_blend(texel_color, toon_color)
                }
              }
            }
          }
        } else {
          color = Some(vertex_color);
        }

        if let Some(mut color) = color {
          let fb_color = pixel.color.unwrap_or(Color::new());
          let fb_alpha = fb_color.alpha.unwrap_or(0x1f);

          let polygon_alpha = color.alpha.unwrap();

          if Self::check_polygon_depth(
            polygon,
            pixel.depth,
            z as u32,
            prev_attributes
          ) {
            if disp3dcnt.contains(Display3dControlRegister::ALPHA_BLENDING_ENABLE) && pixel.color.is_some() && fb_alpha != 0 && polygon_alpha != 0x1f {
              let pixel_color = pixel.color.unwrap().to_rgb6();
              let mut color = Self::blend_colors3d(pixel_color, color, 0x1f - polygon_alpha as u16, (polygon_alpha + 1) as u16);

              color.alpha = Some(cmp::max(fb_alpha, polygon_alpha));

              color.to_rgb5();

              pixel.color = Some(color);

            } else if polygon_alpha != 0 {
              color.to_rgb5();
              pixel.color = Some(color);
              pixel.depth = z as u32;
            }

            if polygon_alpha != 0x1f && polygon.attributes.contains(PolygonAttributes::UPDATE_DEPTH_FOR_TRANSLUCENT) {
              pixel.depth = z as u32;
            }

            attributes_buffer[(x + y * SCREEN_WIDTH as u32) as usize] = RenderingAttributes {
              is_translucent: polygon_alpha != 0x1f,
              front_facing: polygon.is_front,
              fog_enabled: polygon.attributes.contains(PolygonAttributes::FOG_ENABLE)
            };

            if disp3dcnt.contains(Display3dControlRegister::FOG_MASTER_ENABLE) &&
              polygon.attributes.contains(PolygonAttributes::FOG_ENABLE)
            {
              let mut pixel_color = pixel.color.unwrap();

              Self::apply_fog(
                &mut pixel_color.to_rgb6(),
                fog_color,
                fog_offset,
                fog_table,
                disp3dcnt,
                z as u32
              );

              pixel.color = Some(pixel_color.to_rgb5());
            }
          }
        }
        z += dzdx;
        x += 1;
      }
      boundary1 += left_slope;
      boundary2 += right_slope;
      y += 1;
    }
  }

  fn apply_fog(
    color: &mut Color,
    fog_color: &mut Color,
    fog_offset: u16,
    fog_table: &[u8],
    disp3dcnt: &Display3dControlRegister,
    z: u32
  ) {
    let fog_step = 0x400 << disp3dcnt.fog_depth_shift();

    let n = if z < fog_offset as u32 {
      (((z - fog_offset as u32) / fog_step - 1) >> 17) & 0x1f
    } else {
      0
    };

    let mut density = fog_table[n as usize];

    if density == 127 {
      density = 128;
    }

    fog_color.to_rgb6();

    // apply fog formula

    if !disp3dcnt.contains(Display3dControlRegister::FOG_ALPHA_ONLY) {
      color.r = ((fog_color.r as u16 * density as u16 + color.r as u16 * (128 - density as u16)) / 128) as u8;
      color.g = ((fog_color.g as u16 * density as u16 + color.g as u16 * (128 - density as u16)) / 128) as u8;
      color.b = ((fog_color.b as u16 * density as u16 + color.b as u16 * (128 - density as u16)) / 128) as u8;
    }

    color.alpha = Some(
      ((fog_color.alpha6() as u16 * density as u16 + color.alpha6() as u16 * (128 - density as u16)) / 128) as u8
    );
  }

  fn apply_polygon_shading(blended: Color, toon_color: Color) -> Option<Color> {
    let r = (blended.r + toon_color.r).min(0x3f);
    let g = (blended.g + toon_color.g).min(0x3f);
    let b = (blended.b + toon_color.b).min(0x3f);
    let alpha = (blended.alpha.unwrap_or(0x1f) + toon_color.alpha.unwrap_or(0x1f)).min(0x1f);

    Some(Color {
      r,
      g,
      b,
      alpha: Some(alpha)
    })
  }

  fn check_polygon_depth(
    polygon: &Polygon,
    current_depth: u32,
    new_depth: u32,
    prev_attributes: RenderingAttributes
  ) -> bool {
    if polygon.attributes.contains(PolygonAttributes::DRAW_PIXELS_WITH_DEPTH) {
      new_depth >= current_depth - 0x200 && new_depth <= current_depth + 0x200
    } else if !polygon.is_front {
      new_depth < current_depth
    } else {
      if !prev_attributes.is_translucent && !prev_attributes.front_facing {
        new_depth <= current_depth
      } else {
        new_depth < current_depth
      }
    }
  }

  pub fn blend_colors3d(color: Color, color2: Color, eva: u16, evb: u16) -> Color {
    let r = ((color.r as u16 * eva + color2.r as u16 * evb) >> 5).min(0x3f) as u8;
    let g = ((color.g as u16 * eva + color2.g as u16 * evb) >> 5).min(0x3f) as u8;
    let b = ((color.b as u16 * eva + color2.b as u16 * evb) >> 5).min(0x3f) as u8;

    Color {
      r,
      g,
      b,
      alpha: None
    }
  }

  fn modulation_blend(texel: Color, pixel: Color) -> Option<Color> {
    // ((val1 + 1) * (val2 + 1) - 1) / 64;
    let modulation_fn = |component1, component2| ((component1 + 1) * (component2 + 1) - 1) / 64;

    let r = modulation_fn(texel.r as u16, pixel.r as u16) as u8;
    let g = modulation_fn(texel.g as u16, pixel.g as u16) as u8;
    let b = modulation_fn(texel.b as u16, pixel.b as u16) as u8;

    let new_alpha = if pixel.alpha.is_some() && texel.alpha.is_some() {
      Some((modulation_fn(texel.alpha6() as u16, pixel.alpha6() as u16) >> 1) as u8)
    } else {
      texel.alpha
    };

    Some(Color {
      r,
      g,
      b,
      alpha: new_alpha
    })
  }

  fn check_if_texture_repeated(val: i32, repeat: bool, flip: bool, mask: u32, shift: u32) -> u32 {
    let mut return_val = val as u32;
    if repeat {
      return_val &= mask;
      if flip && (val as u32 >> shift) % 2 == 1 {
        return_val ^= mask;
      }
    } else if val < 0 {
      return_val = 0;
    } else if val as u32 >= mask {
      return_val = mask;
    }

    return_val
  }

  fn get_texel_color(polygon: &Polygon, curr_u: i32, curr_v: i32, vram: &VRam, debug_on: bool, found: &mut HashSet<String>) -> Option<Color> {
    let u = curr_u;

    let u = Self::check_if_texture_repeated(
      u,
      polygon.tex_params.repeat_s,
      polygon.tex_params.flip_s,
      polygon.tex_params.texture_s_size - 1,
      polygon.tex_params.size_s_shift
    );

    let v = curr_v;

    let v = Self::check_if_texture_repeated(
      v,
      polygon.tex_params.repeat_t,
      polygon.tex_params.flip_t,
      polygon.tex_params.texture_t_size -1,
      polygon.tex_params.size_t_shift
    );

    // println!("got {u},{v}");

    let texel = u + v * polygon.tex_params.texture_s_size;
    let vram_offset = polygon.tex_params.vram_offset;

    let address = vram_offset + texel;

    let palette_base = polygon.palette_base;

    // println!("vram offset = {:x} palette base = {:x}", vram_offset, palette_base);

    match polygon.tex_params.texture_format {
      TextureFormat::None => None,
      TextureFormat::A3I5Translucent => {
        let byte = vram.read_texture::<u8>(address);

        let palette_index = byte & 0x1f;
        let alpha = (byte >> 5) & 0x7;

        Self::get_palette_color(polygon, palette_base as u32, palette_index as u32, vram, Some(alpha * 4 + alpha / 2))
      }
      TextureFormat::A5I3Translucent => {
        let byte = vram.read_texture::<u8>(address);

        let palette_index = byte & 0x7;

        let alpha = (byte >> 3) & 0x1f;

        Self::get_palette_color(polygon, palette_base as u32, palette_index as u32, vram, Some(alpha))
      }
      TextureFormat::Color16 => {
        let real_address = vram_offset + texel / 2;

        let byte = vram.read_texture::<u8>(real_address);

        let palette_index = if texel & 0b1 == 0 {
          byte & 0xf
        } else {
          (byte >> 4) & 0xf
        };

        Self::get_palette_color(polygon, palette_base as u32, palette_index as u32, vram, Some(0x1f))
      }
      TextureFormat::Color256 => {
        let palette_index = vram.read_texture::<u8>(address);

        Self::get_palette_color(polygon, palette_base as u32, palette_index as u32, vram, Some(0x1f))
      }
      TextureFormat::Color4x4 => {
        let blocks_per_row = polygon.tex_params.texture_s_size / 4;

        let block_address = (u / 4) + blocks_per_row * (v / 4);

        let base_address = vram_offset + 4 * block_address;

        let mut texel_value = vram.read_texture::<u8>(base_address + (v & 0x3));

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

        let extra_palette_info = vram.read_texture::<u16>(slot1_address);

        let palette_offset = palette_base as u32 + ((extra_palette_info & 0x3fff) * 4) as u32;

        let mode = (extra_palette_info >> 14) & 0x3;

        let get_color = |num: u32| {
          let mut color = Color::from(
            vram.read_texture_palette(palette_offset + 2 * num)
          );
          color.alpha = Some(0x1f);

          color
        };

        match (texel_value, mode) {
          (0, _) => Some(get_color(0).to_rgb6()),
          (1, _) => Some(get_color(1).to_rgb6()),
          (2, 0) | (2, 2) => Some(get_color(2).to_rgb6()),
          (2, 1) => {
            // (color0 + color1) / 2
            let color0 = get_color(0);
            let color1 = get_color(1);

            let mut blended_color = color0.blend_half(color1);

            Some(blended_color.to_rgb6())
          }
          (2, 3) => {
            // (color0 * 5 + color1 * 3) / 8

            let color0 = get_color(0);
            let color1 = get_color(1);

            let mut blended_color = color0.blend_texture(color1);

            Some(blended_color.to_rgb6())
          }
          (3, 0)| (3, 1) => Some(Color { r: 0, g: 0, b: 0, alpha: Some(0) }), // transparent
          (3, 2) => Some(get_color(3).to_rgb6()),
          (3, 3) => {
            // (color0 * 3 + color1 * 5) / 8
            let color0 = get_color(0);
            let color1 = get_color(1);

            let mut blended_color = color1.blend_texture(color0);

            Some(blended_color.to_rgb6())
          }
          _ => panic!("invalid options given for texel value and mode: {texel_value} {mode}")
        }
      }
      TextureFormat::Color4 => {
        let mut palette_index = vram.read_texture::<u8>(vram_offset + texel / 4);

        palette_index = match texel & 0x3 {
          0 => palette_index & 0x3,
          1 => (palette_index >> 2) & 0x3,
          2 => (palette_index >> 4) & 0x3,
          3 => (palette_index >> 6) & 0x3,
          _ => unreachable!()
        };

        let address = palette_base as u32 / 2 + palette_index as u32 * 2;
        let color_raw = vram.read_texture_palette(address);

        let alpha = if palette_index == 0 && polygon.tex_params.color0_transparent {
          Some(0)
        } else {
          Some(0x1f)
        };

        let mut color = Color::from(color_raw);
        color.alpha = alpha;

        Some(color)
      }
      TextureFormat::Direct => {
        let address = vram_offset + 2 * texel;
        let color_raw = vram.read_texture::<u16>(address);

        let alpha = if color_raw & 0x8000 == 0 { Some(0) } else { Some(0x1f) };

        let mut color = Color::from(color_raw);

        color.alpha = alpha;

        Some(color)
      }
    }
  }
}