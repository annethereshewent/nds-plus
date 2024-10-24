use crate::gpu::
{engine_3d::Pixel3d, registers::
  {
    bg_control_register::BgControlRegister,
    display_control_register::
    {
      BgMode,
      DisplayControlRegisterFlags,
      DisplayMode
    }
  }, vram::VRam, SCREEN_HEIGHT, SCREEN_WIDTH
};

#[derive(PartialEq, Copy, Clone)]
enum AffineType {
  Extended8bpp,
  Extended8bppDirect,
  Extended,
  Normal,
  Large
}

use super::{Color, Engine2d, OamAttributes, ObjectPixel, AFFINE_SIZE, ATTRIBUTE_SIZE, COLOR_TRANSPARENT, OBJ_PALETTE_OFFSET};

impl<const IS_ENGINE_B: bool> Engine2d<IS_ENGINE_B> {
  fn render_affine_object(&mut self, obj_attributes: OamAttributes, y: u16, vram: &VRam) {
    let (obj_width, obj_height) = obj_attributes.get_object_dimensions();

    let (x_coordinate, y_coordinate) = self.get_obj_coordinates(obj_attributes.x_coordinate, obj_attributes.y_coordinate);

    let (bbox_width, bbox_height) = if obj_attributes.double_sized_flag {
      (2 * obj_width, 2 * obj_height)
    } else {
      (obj_width, obj_height)
    };

    let y_pos_in_sprite = y as i16 - y_coordinate;

    if y_pos_in_sprite < 0 || y_pos_in_sprite as u32 >= bbox_height {
      return;
    }

    let tile_number = obj_attributes.tile_number;

    let palette_bank = if !obj_attributes.palette_flag {
      obj_attributes.palette_number
    } else {
      0
    };

    // get affine matrix
    let (dx, dmx, dy, dmy) = self.get_obj_affine_params(obj_attributes.rotation_param_selection);

    let y_offset = bbox_height / 2;
    let x_offset: i16 = bbox_width as i16 / 2;

    let iy = y as i16 - (y_coordinate + y_offset as i16);

    for ix in (-x_offset)..(x_offset) {
      let x = x_coordinate + x_offset + ix;

      if x < 0 {
        continue;
      }

      if x as u16 >= SCREEN_WIDTH {
        break;
      }

      if self.obj_lines[x as usize].priority <= obj_attributes.priority && obj_attributes.obj_mode != 2 {
        continue;
      }

      let transformed_x = (dx * ix + dmx * iy) >> 8;
      let transformed_y = (dy * ix + dmy * iy) >> 8;

      let texture_x = transformed_x + obj_width as i16 / 2;
      let texture_y = transformed_y + obj_height as i16 / 2;

      if texture_x >= 0 && texture_x < obj_width as i16 && texture_y >= 0 && texture_y < obj_height as i16 {
        // finally queue the pixel!

        if obj_attributes.obj_mode == 3 {
          self.render_bitmap_object(x as usize, texture_x as u32, texture_y as u32, obj_width, &obj_attributes, vram);
        } else {
          let tile_x = texture_x % 8;
          let tile_y = texture_y % 8;

          let bit_depth = if obj_attributes.palette_flag {
            8
          } else {
            4
          };

          let (boundary, offset) = self.get_boundary_and_offset(texture_x as u32, texture_y as u32, bit_depth, obj_width);

          let tile_address = tile_number as u32 * boundary + offset * bit_depth * 8;

          let palette_index = if obj_attributes.palette_flag {
            self.get_obj_pixel_index_bpp8(tile_address, tile_x as u16, tile_y as u16, false, false, vram)
          } else {
            self.get_obj_pixel_index_bpp4(tile_address, tile_x as u16, tile_y as u16, false, false, vram)
          };

          if palette_index != 0 {
            let color = if bit_depth == 8 && self.dispcnt.flags.contains(DisplayControlRegisterFlags::OBJ_EXTENDED_PALETTES) {
              self.get_obj_extended_palette(palette_index as u32, obj_attributes.palette_number as u32, vram)
            } else {
              self.get_obj_palette_color(palette_index as usize, palette_bank as usize)
            };

            self.obj_lines[x as usize] = ObjectPixel {
              priority: obj_attributes.priority,
              color,
              is_window: obj_attributes.obj_mode == 2,
              is_transparent: obj_attributes.obj_mode == 1
            }
          }
        }
      }
    }
  }

  pub fn render_normal_line(&mut self, y: u16, vram: &VRam, frame_buffer: &[Pixel3d]) {
    if self.dispcnt.flags.contains(DisplayControlRegisterFlags::DISPLAY_OBJ) {
      self.render_objects(y, vram);
    }

    if self.bg_mode_enabled(0) {
      if !IS_ENGINE_B && (self.dispcnt.bg_mode == BgMode::Mode6 || self.dispcnt.flags.contains(DisplayControlRegisterFlags::BG_3D_SELECTION)) {
        self.render_3d_line(y, frame_buffer);
      } else {
        self.render_text_line(0, y, vram);
      }
    }

    match self.dispcnt.bg_mode {
      BgMode::Mode0 => {
        for i in 1..4 {
          if self.bg_mode_enabled(i) {
            self.render_text_line(i, y, vram);
          }
        }
      }
      BgMode::Mode1 => {
        for i in 1..3 {
          if self.bg_mode_enabled(i) {
            self.render_text_line(i, y, vram);
          }
        }

        if self.bg_mode_enabled(3) {
          self.render_affine_line(3, vram, AffineType::Normal);
        }
      }
      BgMode::Mode2 => {
        for i in 1..2 {
          if self.bg_mode_enabled(i) {
            self.render_text_line(i, y, vram);
          }
        }

        for i in 2..4 {
          if self.bg_mode_enabled(i) {
            self.render_affine_line(i, vram, AffineType::Normal);
          }
        }
      }
      BgMode::Mode3 => {
        for i in 1..3 {
          if self.bg_mode_enabled(i) {
            self.render_text_line(i, y, vram);
          }
        }

        if self.bg_mode_enabled(3) {
          self.render_extended_line(3, vram);
        }
      }
      BgMode::Mode4 => {
        if self.bg_mode_enabled(1) {
          self.render_text_line(1, y, vram);
        }


        if self.bg_mode_enabled(2) {
          self.render_affine_line(2, vram, AffineType::Normal);
        }

        if self.bg_mode_enabled(3) {
          self.render_extended_line(3, vram);
        }
      }
      BgMode::Mode5 => {
        if self.bg_mode_enabled(1) {
          self.render_text_line(1, y, vram);
        }

        if self.bg_mode_enabled(2) {
          self.render_extended_line(2, vram);
        }

        if self.bg_mode_enabled(3) {
          self.render_extended_line(3, vram);
        }
      }
      BgMode::Mode6 => {
        self.render_affine_line(2, vram, AffineType::Large)
      }
    }

    self.finalize_scanline(y);
  }

  fn render_extended_line(&mut self, bg_index: usize, vram: &VRam) {
    if self.bgcnt[bg_index].contains(BgControlRegister::PALETTES) {
      if self.bgcnt[bg_index].character_base_block() & 0b1 != 0 {
        // Extended Direct
        self.render_affine_line(bg_index, vram, AffineType::Extended8bppDirect);
      } else {
        // Extended8bpp
        self.render_affine_line(bg_index, vram, AffineType::Extended8bpp);
      }
    } else {
      // Extended
      self.render_affine_line(bg_index, vram, AffineType::Extended);
    }
  }

  fn render_3d_line(&mut self, y: u16, frame_buffer: &[Pixel3d]) {

    for x in 0..SCREEN_WIDTH {
      let pixel = frame_buffer[(x + y * SCREEN_WIDTH) as usize];

      self.bg_lines[0][x as usize] = pixel.color;
    }
  }

  fn render_objects(&mut self, y: u16, vram: &VRam) {
    for i in 0..128 {
      let obj_attributes = self.get_attributes(i);

      if obj_attributes.obj_disable {
        continue;
      }
      if obj_attributes.rotation_flag {
        self.render_affine_object(obj_attributes, y, vram);
      } else {
        self.render_normal_object(obj_attributes, y, vram);
      }
    }
  }

  fn get_obj_affine_params(&self, affine_index: u16) -> (i16, i16, i16, i16) {
    let mut offset = affine_index * 32 + AFFINE_SIZE;

    let dx = self.oam_read_16(offset as usize) as i16;
    offset += 2 + AFFINE_SIZE;
    let dmx = self.oam_read_16(offset as usize) as i16;
    offset += 2 + AFFINE_SIZE;
    let dy = self.oam_read_16(offset as usize) as i16;
    offset += 2 + AFFINE_SIZE;
    let dmy = self.oam_read_16(offset as usize) as i16;

    (dx, dmx, dy, dmy)
  }

  fn get_attributes(&self, i: usize) -> OamAttributes {
    let oam_address = i * ATTRIBUTE_SIZE;

    let attribute1 = self.oam_read_16(oam_address);
    let attribute2 = self.oam_read_16(oam_address + 2);
    let attribute3 = self.oam_read_16(oam_address + 4);

    let y_coordinate = attribute1 & 0xff;
    let rotation_flag = (attribute1 >> 8) & 0b1 == 1;
    let double_sized_flag = rotation_flag && (attribute1 >> 9) & 0b1 == 1;
    let obj_disable = !rotation_flag && (attribute1 >> 9) & 0b1 == 1;
    let obj_mode = (attribute1 >> 10) & 0x3;
    let obj_mosaic = (attribute1 >> 12) & 0b1 == 1;
    let palette_flag = (attribute1 >> 13) & 0b1 == 1;
    let obj_shape = (attribute1 >> 14) & 0x3;

    let x_coordinate = attribute2 & 0x1ff;
    let rotation_param_selection = if rotation_flag {
      (attribute2 >> 9) & 0x1f
    } else {
      0
    };
    let horizontal_flip = !rotation_flag && (attribute2 >> 12) & 0b1 == 1;
    let vertical_flip = !rotation_flag && (attribute2 >> 13) & 0b1 == 1;
    let obj_size = (attribute2 >> 14) & 0b11;

    let tile_number = attribute3 & 0x3ff;
    let priority = (attribute3 >> 10) & 0x3;
    let palette_number = (attribute3 >> 12) & 0xf;

    OamAttributes {
      y_coordinate,
      rotation_flag,
      double_sized_flag,
      obj_disable,
      obj_mode,
      _obj_mosaic: obj_mosaic,
      palette_flag,
      obj_shape,
      x_coordinate,
      rotation_param_selection,
      horizontal_flip,
      vertical_flip,
      obj_size,
      tile_number,
      priority,
      palette_number
    }

  }

  pub fn bg_mode_enabled(&self, bg_index: usize) -> bool {
    match bg_index {
      0 => self.dispcnt.flags.contains(DisplayControlRegisterFlags::DISPLAY_BG0),
      1 => self.dispcnt.flags.contains(DisplayControlRegisterFlags::DISPLAY_BG1),
      2 => self.dispcnt.flags.contains(DisplayControlRegisterFlags::DISPLAY_BG2),
      3 => self.dispcnt.flags.contains(DisplayControlRegisterFlags::DISPLAY_BG3),
      _ => unreachable!("can't happen")
    }
  }

  fn render_text_line(&mut self, bg_index: usize, y: u16, vram: &VRam) {
    let (x_offset, y_offset) = (self.bgxofs[bg_index], self.bgyofs[bg_index]);
    /*
      engine A screen base: BGxCNT.bits*2K + DISPCNT.bits*64K
      engine B screen base: BGxCNT.bits*2K + 0
      engine A char base: BGxCNT.bits*16K + DISPCNT.bits*64K
      engine B char base: BGxCNT.bits*16K + 0
     */
    let (tilemap_base, tile_base) = self.get_tile_base_addresses(bg_index);

    let mut x = 0;

    let (background_width, background_height) = self.bgcnt[bg_index].get_screen_dimensions();

    let x_in_bg = (x + x_offset) % background_width;
    let y_in_bg = (y + y_offset) % background_height;

    let mut x_tile_number = (x_in_bg as u32 / 8) % 32;

    let y_tile_number = (y_in_bg as u32 / 8) % 32;

    let mut x_pos_in_tile = x_in_bg % 8;
    let y_pos_in_tile = y_in_bg % 8;

    let mut screen_index = match self.bgcnt[bg_index].screen_size() {
      0 => 0,
      1 => x_in_bg / 256, // 512 x 256
      2 => y_in_bg / 256, // 256 x 512
      3 => (x_in_bg / 256) + ((y_in_bg / 256) * 2), // 512 x 512
      _ => unreachable!("not possible")
    };

    let is_bpp8 = self.bgcnt[bg_index].contains(BgControlRegister::PALETTES);

    let bit_depth = if is_bpp8 { 8 } else { 4 };

    while x < SCREEN_WIDTH {
      let tilemap_number = x_tile_number + y_tile_number * 32;
      let mut tilemap_address = tilemap_base + 0x800 * screen_index as u32 + 2 * tilemap_number as u32;
      'outer: for _ in x_tile_number..32 {
        let attributes = if !IS_ENGINE_B {
          vram.read_engine_a_bg::<u16>(tilemap_address)
        } else {
          vram.read_engine_b_bg::<u16>(tilemap_address)
        };

        let x_flip = (attributes >> 10) & 0x1 == 1;
        let y_flip =  (attributes >> 11) & 0x1 == 1;
        let palette_number = (attributes >> 12) & 0xf;
        let tile_number = attributes & 0x3ff;

        let tile_address = tile_base + tile_number as u32 * bit_depth * 8;

        for tile_x in x_pos_in_tile..8 {
          let palette_index = if bit_depth == 8 {
            self.get_bg_pixel_index_bpp8(tile_address, tile_x, y_pos_in_tile, x_flip, y_flip, vram)
          } else {
            self.get_bg_pixel_index_bpp4(tile_address, tile_x, y_pos_in_tile, x_flip, y_flip, vram)
          };

          let palette_bank = if is_bpp8 {
            0
          } else {
            palette_number
          };

          self.bg_lines[bg_index][x as usize] = if is_bpp8 && self.dispcnt.flags.contains(DisplayControlRegisterFlags::BG_EXTENDED_PALETTES) {
            self.get_bg_extended_palette_color(bg_index, palette_index as usize, palette_number as usize, vram)
          } else {
            self.get_bg_palette_color(palette_index as usize, palette_bank as usize)
          };

          x += 1;

          if x == SCREEN_WIDTH {
            break 'outer;
          }
        }
        x_pos_in_tile = 0;
        tilemap_address += 2;
      }
      x_tile_number = 0;
      if background_width == 512 {
        screen_index ^= 1;
      }
    }
  }

  fn get_boundary_and_offset(&self, x_pos_in_sprite: u32, y_pos_in_sprite: u32, bit_depth: u32, obj_width: u32) -> (u32, u32) {
    if !self.dispcnt.flags.contains(DisplayControlRegisterFlags::TILE_OBJ_MAPPINGS) {
      (
        32 as u32,
        y_pos_in_sprite as u32 / 8 * 0x80 / (bit_depth as u32) + (x_pos_in_sprite  as u32) / 8,
      )
    } else {
      (
        32 << self.dispcnt.tile_obj_boundary as u32,
        (y_pos_in_sprite as u32 / 8 * obj_width + x_pos_in_sprite) / 8,
      )
    }
  }

  fn get_palette_color(&self, index: usize, palette_bank: usize, offset: usize) -> Option<Color> {
    let value = if index == 0 || (palette_bank != 0 && index % 16 == 0) {
      COLOR_TRANSPARENT
    } else {
      let index = 2 * index + 32 * palette_bank + offset;

      let lower = self.palette_ram[index];
      let upper = self.palette_ram[index + 1];

      (lower as u16) | (upper as u16) << 8
    };

    if value == COLOR_TRANSPARENT {
      None
    } else {
      Some(Color::from(value))
    }
  }

  fn get_bg_extended_palette_color(&self, bg_index: usize, palette_index: usize, palette_bank: usize, vram: &VRam) -> Option<Color> {
    let slot = if bg_index < 2 && self.bgcnt[bg_index].contains(BgControlRegister::DISPLAY_AREA_OVERFLOW) {
      bg_index + 2
    } else {
      bg_index
    };

    let address = slot as u32 * 8 * 0x400 + (palette_index + palette_bank * 256) as u32 * 2;

    let color_raw = if !IS_ENGINE_B {
      vram.read_engine_a_extended_bg_palette::<u16>(address)
    } else {
      vram.read_engine_b_extended_bg_palette::<u16>(address)
    };

    if color_raw != COLOR_TRANSPARENT && palette_index != 0 {
      Some(Color::from(color_raw))
    } else {
      None
    }
  }

  fn get_bg_palette_color(&self, index: usize, palette_bank: usize) -> Option<Color> {
    self.get_palette_color(index, palette_bank, 0)
  }

  fn get_obj_palette_color(&self, index: usize, palette_bank: usize) -> Option<Color> {
    self.get_palette_color(index, palette_bank, OBJ_PALETTE_OFFSET)
  }

  fn get_obj_extended_palette(&self, index: u32, palette_bank: u32, vram: &VRam) -> Option<Color> {
    let address = (palette_bank * 256 + index) * 2;

    let color = if !IS_ENGINE_B {
      vram.read_engine_a_extended_obj_palette(address)
    } else {
      vram.read_engine_b_extended_obj_palette(address)
    };

    if color != COLOR_TRANSPARENT && index != 0 {
      Some(Color::from(color))
    } else {
      None
    }
  }

  fn get_obj_pixel_index_bpp8(&self, address: u32, tile_x: u16, tile_y: u16, x_flip: bool, y_flip: bool, vram: &VRam) -> u8 {
    let tile_x = if x_flip { 7 - tile_x } else { tile_x };
    let tile_y = if y_flip { 7 - tile_y } else { tile_y };

    if !IS_ENGINE_B {
      vram.read_engine_a_obj(address + tile_x as u32 + (tile_y as u32) * 8)
    } else {
      vram.read_engine_b_obj(address + tile_x as u32 + (tile_y as u32) * 8)
    }
  }

  fn get_obj_pixel_index_bpp4(&self, address: u32, tile_x: u16, tile_y: u16, x_flip: bool, y_flip: bool, vram: &VRam) -> u8 {
    let tile_x = if x_flip { 7 - tile_x } else { tile_x };
    let tile_y = if y_flip { 7 - tile_y } else { tile_y };

    let address = address + (tile_x / 2) as u32 + (tile_y as u32) * 4;

    let byte = if !IS_ENGINE_B {
      vram.read_engine_a_obj::<u8>(address)
    } else {
      vram.read_engine_b_obj::<u8>(address)
    };

    if tile_x & 0b1 == 1 {
      byte >> 4
    } else {
      byte & 0xf
    }
  }

  fn get_bg_pixel_index_bpp8(&self, address: u32, tile_x: u16, tile_y: u16, x_flip: bool, y_flip: bool, vram: &VRam) -> u8 {
    let tile_x = if x_flip { 7 - tile_x } else { tile_x };
    let tile_y = if y_flip { 7 - tile_y } else { tile_y };

    if !IS_ENGINE_B {
      vram.read_engine_a_bg(address + tile_x as u32 + (tile_y as u32) * 8)
    } else {
      vram.read_engine_b_bg(address + tile_x as u32 + (tile_y as u32) * 8)
    }
  }

  fn get_bg_pixel_index_bpp4(&self, address: u32, tile_x: u16, tile_y: u16, x_flip: bool, y_flip: bool, vram: &VRam) -> u8 {
    let tile_x = if x_flip { 7 - tile_x } else { tile_x };
    let tile_y = if y_flip { 7 - tile_y } else { tile_y };

    let address = address + (tile_x / 2) as u32 + (tile_y as u32) * 4;

    let byte = if !IS_ENGINE_B {
      vram.read_engine_a_bg::<u8>(address)
    } else {
      vram.read_engine_b_bg::<u8>(address)
    };

    if tile_x & 0b1 == 1 {
      byte >> 4
    } else {
      byte & 0xf
    }
  }

  fn get_obj_coordinates(&mut self, x: u16, y: u16) -> (i16, i16) {
    let return_x: i16 = if x >= SCREEN_WIDTH {
      x as i16 - 512
    } else {
      x as i16
    };

    let return_y: i16 = if y >= SCREEN_HEIGHT {
      y as i16 - 256
    } else {
      y as i16
    };

    (return_x, return_y)
  }

  fn render_bitmap_object(&mut self, x: usize, x_pos_in_sprite: u32, y_pos_in_sprite: u32, obj_width: u32, obj_attributes: &OamAttributes, vram: &VRam) {
    // 1d object
    let (tile_base, width) = if self.dispcnt.flags.contains(DisplayControlRegisterFlags::BITMAP_OBJ_MAPPING) {
      // means the object is a square which isnt allowed in 1d mode
      if self.dispcnt.flags.contains(DisplayControlRegisterFlags::BITMAP_OBJ_2D_DIMENSION) {
        return;
      }
      let boundary = if self.dispcnt.flags.contains(DisplayControlRegisterFlags::BITMAP_OBJ_1D_BOUNDARY) {
        256
      } else {
        128
      };

      (obj_attributes.tile_number * boundary, obj_width)
    } else {
      let mut mask_x = 0xf;
      let mut width = 128;

      if self.dispcnt.flags.contains(DisplayControlRegisterFlags::BITMAP_OBJ_2D_DIMENSION) {
        mask_x = 0x1f;
        width = 256;
      }

      // 2D_BitmapVramAddress = (TileNo AND MaskX)*10h + (TileNo AND NOT MaskX)*80h
      ((obj_attributes.tile_number & mask_x) * 0x10 + (obj_attributes.tile_number & !mask_x) * 0x80, width)
    };

    let tile_address = tile_base as u32 + 2 * (x_pos_in_sprite + y_pos_in_sprite * width);

    let color_raw = if !IS_ENGINE_B {
      vram.read_engine_a_obj::<u16>(tile_address)
    } else {
      vram.read_engine_b_obj::<u16>(tile_address)
    };

    let color = if color_raw == 0 {
      None
    } else {
      Some(Color::from(color_raw))
    };

    self.obj_lines[x] = ObjectPixel {
      priority: obj_attributes.priority,
      color,
      is_window: obj_attributes.obj_mode == 2,
      is_transparent: obj_attributes.obj_mode == 1
    };
  }

  fn render_normal_object(&mut self, obj_attributes: OamAttributes, y: u16, vram: &VRam) {
    let (obj_width, obj_height) = obj_attributes.get_object_dimensions();

    let (x_coordinate, y_coordinate) = self.get_obj_coordinates(obj_attributes.x_coordinate, obj_attributes.y_coordinate);

    let y_pos_in_sprite = y as i16 - y_coordinate;

    if y_pos_in_sprite < 0 || y_pos_in_sprite as u32 >= obj_height {
      return;
    }

    let tile_number = obj_attributes.tile_number;

    let bit_depth = if obj_attributes.palette_flag {
      8
    } else {
      4
    };

    let mut palette_bank = if !obj_attributes.palette_flag {
      obj_attributes.palette_number
    } else {
      0
    };

    for x in 0..obj_width {
      let screen_x = x as i16 + x_coordinate;

      if screen_x < 0 {
        continue;
      }

      if screen_x >= SCREEN_WIDTH as i16 {
        break;
      }

      if self.obj_lines[screen_x as usize].priority <= obj_attributes.priority && obj_attributes.obj_mode != 2 {
        continue;
      }

      let x_pos_in_sprite = if obj_attributes.horizontal_flip {
        obj_width - x - 1
      } else {
        x
      };

      let y_pos_in_sprite = if obj_attributes.vertical_flip {
        (obj_height as i16 - y_pos_in_sprite - 1) as u16
      } else {
        y_pos_in_sprite as u16
      };

      if obj_attributes.obj_mode == 3 {
        self.render_bitmap_object(screen_x as usize, x_pos_in_sprite, y_pos_in_sprite as u32, obj_width, &obj_attributes, vram);
      } else {
        let x_pos_in_tile = x_pos_in_sprite % 8;
        let y_pos_in_tile = y_pos_in_sprite % 8;

        let (boundary, offset) = self.get_boundary_and_offset(x_pos_in_sprite as u32, y_pos_in_sprite as u32, bit_depth, obj_width);

        let tile_address = tile_number as u32 * boundary + offset * bit_depth * 8;

        let palette_index = if bit_depth == 8 {
          self.get_obj_pixel_index_bpp8(tile_address, x_pos_in_tile as u16, y_pos_in_tile, false, false, &vram)
        } else {
          self.get_obj_pixel_index_bpp4(tile_address, x_pos_in_tile as u16, y_pos_in_tile, false, false, &vram)
        };


        if palette_index != 0 {
          // need to determine whether to look at extended palette or regular obj palette
          let color = if bit_depth == 8 && self.dispcnt.flags.contains(DisplayControlRegisterFlags::OBJ_EXTENDED_PALETTES) {
            self.get_obj_extended_palette(palette_index as u32, obj_attributes.palette_number as u32, vram)
          } else {
            if bit_depth == 8 {
              palette_bank = 0;
            }
            self.get_obj_palette_color(palette_index as usize, palette_bank as usize)
          };
          self.obj_lines[screen_x as usize] = ObjectPixel {
            priority: obj_attributes.priority,
            color,
            is_window: obj_attributes.obj_mode == 2,
            is_transparent: obj_attributes.obj_mode == 1
          };
        }
      }
    }
  }

  pub fn render_line(&mut self, y: u16, vram: &mut VRam, frame_buffer: &[Pixel3d]) {
    match self.dispcnt.display_mode {
      DisplayMode::Mode0 => {
        let color = Color {
          r: 0xff,
          g: 0xff,
          b: 0xff,
          alpha: None
        };

        for x in 0..SCREEN_WIDTH {
          self.set_pixel(x as usize, y as usize, color);
        }
      },
      DisplayMode::Mode1 => self.render_normal_line(y, vram, frame_buffer),
      DisplayMode::Mode2 => {
        for x in 0..SCREEN_WIDTH {
          let index = 2 * (y as usize * SCREEN_WIDTH as usize + x as usize);
          let bank = vram.get_lcdc_bank(self.dispcnt.vram_block);

          let color = bank[index] as u16 | (bank[(index + 1) as usize] as u16) << 8;

          let color = Color::to_rgb24(color);

          self.set_pixel(x as usize, y as usize, color);
        }
      }
      DisplayMode::Mode3 => todo!()
    }
  }

  fn oam_read_16(&self, address: usize) -> u16 {
    (self.oam[address] as u16) | (self.oam[address + 1] as u16) << 8
  }

  pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
    let i = 4 * (x + y * SCREEN_WIDTH as usize);

    self.pixels[i] = color.r;
    self.pixels[i + 1] = color.g;
    self.pixels[i + 2] = color.b;
    self.pixels[i + 3] = 0xff;
  }

  fn get_extended_tilemap_address(tilemap_base: u32, transformed_x: i32, transformed_y: i32, texture_size: i32) -> u32 {
    let x_tile_number = (transformed_x / 8) % (texture_size / 8);
    let y_tile_number = (transformed_y / 8) % (texture_size / 8);

    let tilemap_number = x_tile_number + y_tile_number  * (texture_size / 8);

    tilemap_base + 2 * tilemap_number as u32
  }

  fn get_affine_tilemap_address(tilemap_base: u32, transformed_x: i32, transformed_y: i32, texture_size: i32) -> u32 {
    let x_tile_number = (transformed_x / 8) % (texture_size / 8);
    let y_tile_number = (transformed_y / 8) % (texture_size / 8);

    let tilemap_number = x_tile_number + y_tile_number  * (texture_size / 8);

    tilemap_base + tilemap_number as u32
  }

  fn get_tile_base_addresses(&self, bg_index: usize) -> (u32, u32) {
    if !IS_ENGINE_B {
      (self.bgcnt[bg_index].screen_base_block() as u32 * 0x800 + self.dispcnt.screen_base * 0x1_0000, self.bgcnt[bg_index].character_base_block() as u32 * 0x4000 + self.dispcnt.character_base * 0x1_0000)
    } else {
      (self.bgcnt[bg_index].screen_base_block() as u32 * 0x800, self.bgcnt[bg_index].character_base_block() as u32 * 0x4000)
    }
  }

  fn render_affine_line(&mut self, bg_index: usize, vram: &VRam, affine_type: AffineType) {
    let (dx, dy) = (self.bg_props[bg_index - 2].dx, self.bg_props[bg_index - 2].dy);

    let (tilemap_base, tile_base) = self.get_tile_base_addresses(bg_index);

    let texture_size = if affine_type != AffineType::Large {
      128 << self.bgcnt[bg_index].screen_size()
    } else {
      512 << (self.bgcnt[bg_index].screen_size() & 0b1)
    };

    let (mut ref_x, mut ref_y) = (self.bg_props[bg_index - 2].internal_x, self.bg_props[bg_index - 2].internal_y);

    self.bg_props[bg_index - 2].internal_x += self.bg_props[bg_index - 2].dmx as i32;
    self.bg_props[bg_index - 2].internal_y += self.bg_props[bg_index - 2].dmy as i32;

    for x in 0..SCREEN_WIDTH {
      let mut transformed_x = ref_x >> 8;
      let mut transformed_y = ref_y >> 8;

      ref_x += dx as i32;
      ref_y += dy as i32;

      if transformed_x < 0 || transformed_x >= texture_size || transformed_y < 0 || transformed_y >= texture_size {
        if self.bgcnt[bg_index].contains(BgControlRegister::DISPLAY_AREA_OVERFLOW) {
          transformed_x = transformed_x % texture_size;
          transformed_y = transformed_y % texture_size;
        } else {
          self.bg_lines[bg_index][x as usize] = None;
          continue;
        }
      }

      let x_pos_in_tile = transformed_x % 8;
      let y_pos_in_tile = transformed_y % 8;

      // formulas for extended lines:
      // for extended 8bpp direct color = 2*(transformed_y * texture_size + x);
      // for extended 8bpp, palette_index = transformed_y * WIDTH + x,
      // for extended, get the attributes from vram and render accordingly
      self.bg_lines[bg_index][x as usize] = match affine_type {
        AffineType::Extended => {
          let bit_depth = 8;

          let tilemap_address = Self::get_extended_tilemap_address(tilemap_base, transformed_x, transformed_y, texture_size);

          let attributes = if !IS_ENGINE_B {
            vram.read_engine_a_bg::<u16>(tilemap_address)
          } else {
            vram.read_engine_b_bg::<u16>(tilemap_address)
          };

          let x_flip = (attributes >> 10) & 0x1 == 1;
          let y_flip =  (attributes >> 11) & 0x1 == 1;
          let palette_number = (attributes >> 12) & 0xf;
          let tile_number = attributes & 0x3ff;

          let tile_address = tile_base + tile_number as u32 * bit_depth * 8;

          let palette_index = self.get_bg_pixel_index_bpp8(tile_address, x_pos_in_tile as u16, y_pos_in_tile as u16, x_flip, y_flip, vram);

          if self.dispcnt.flags.contains(DisplayControlRegisterFlags::BG_EXTENDED_PALETTES) {
            self.get_bg_extended_palette_color(bg_index, palette_index as usize, palette_number as usize, vram)
          } else {
            self.get_bg_palette_color(palette_index as usize, 0)
          }
        }
        AffineType::Normal => {
          let bit_depth = 8;

          let tilemap_address = Self::get_affine_tilemap_address(tilemap_base, transformed_x, transformed_y, texture_size);

          let tile_number = if !IS_ENGINE_B {
            vram.read_engine_a_bg::<u8>(tilemap_address)
          } else {
            vram.read_engine_b_bg::<u8>(tilemap_address)
          };

          let tile_address = tile_base + tile_number as u32 * bit_depth as u32 * 8;

          let palette_index = self.get_bg_pixel_index_bpp8(tile_address, x_pos_in_tile as u16, y_pos_in_tile as u16, false, false, vram);

          self.get_bg_palette_color(palette_index as usize, 0)
        }
        AffineType::Extended8bppDirect => {
          let address = 2 * (transformed_y as u32 * texture_size as u32 + x as u32);
          let color_raw = if !IS_ENGINE_B {
            vram.read_engine_a_bg::<u16>(address)
          } else {
            vram.read_engine_b_bg::<u16>(address)
          };

          if color_raw == 0 {
            None
          } else {
            Some(Color::from(color_raw))
          }
        }
        AffineType::Extended8bpp => {
          let palette_address = transformed_y as u32 * SCREEN_WIDTH as u32 + x as u32;

          let palette_index = if !IS_ENGINE_B {
            vram.read_engine_a_bg::<u8>(palette_address)
          } else {
            vram.read_engine_b_bg::<u8>(palette_address)
          };

          self.get_bg_palette_color(palette_index as usize, 0)
        }
        AffineType::Large => {
          let palette_address = transformed_y as u32 * texture_size as u32 + transformed_x as u32;

          let palette_index = if !IS_ENGINE_B {
            vram.read_engine_a_bg::<u8>(palette_address)
          } else {
            vram.read_engine_b_bg::<u8>(palette_address)
          };

          self.get_bg_palette_color(palette_index as usize, 0)
        }
      };
    }
  }

  pub fn on_end_vblank(&mut self) {
    for bg_prop in &mut self.bg_props {
      bg_prop.internal_x = bg_prop.x;
      bg_prop.internal_y = bg_prop.y;
    }
  }
}