use crate::gpu::
{
  engine_3d::Pixel3d,
  registers::
  {
    bg_control_register::BgControlRegister,
    display_control_register::
    {
      BgMode,
      DisplayControlRegisterFlags,
      DisplayMode
    }
  },
  rendering_data::RenderingData,
  vram::VRam,
  SCREEN_HEIGHT,
  SCREEN_WIDTH
};

#[derive(PartialEq, Copy, Clone)]
enum AffineType {
  Extended8bpp,
  Extended8bppDirect,
  Extended,
  Normal,
  Large
}

use super::{renderer2d::Renderer2d, Color, OamAttributes, ObjectPixel, AFFINE_SIZE, ATTRIBUTE_SIZE, COLOR_TRANSPARENT, OBJ_PALETTE_OFFSET};

impl Renderer2d {
  fn render_affine_object(obj_attributes: OamAttributes, y: u16, vram: &VRam, is_engine_b: bool, data: &mut RenderingData) {
    let (obj_width, obj_height) = obj_attributes.get_object_dimensions();

    let (x_coordinate, y_coordinate) = Self::get_obj_coordinates(obj_attributes.x_coordinate, obj_attributes.y_coordinate);

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
    let (dx, dmx, dy, dmy) = Self::get_obj_affine_params(obj_attributes.rotation_param_selection, data);

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

      if data.obj_lines[x as usize].priority <= obj_attributes.priority && obj_attributes.obj_mode != 2 {
        continue;
      }

      let transformed_x = (dx * ix + dmx * iy) >> 8;
      let transformed_y = (dy * ix + dmy * iy) >> 8;

      let texture_x = transformed_x + obj_width as i16 / 2;
      let texture_y = transformed_y + obj_height as i16 / 2;

      if texture_x >= 0 && texture_x < obj_width as i16 && texture_y >= 0 && texture_y < obj_height as i16 {
        // finally queue the pixel!

        if obj_attributes.obj_mode == 3 {
          Self::render_bitmap_object(
            x as usize,
            texture_x as u32,
            texture_y as u32,
            obj_width,
            &obj_attributes,
            vram,
            is_engine_b,
            data
          );
        } else {
          let tile_x = texture_x % 8;
          let tile_y = texture_y % 8;

          let bit_depth = if obj_attributes.palette_flag {
            8
          } else {
            4
          };

          let (boundary, offset) = Self::get_boundary_and_offset(texture_x as u32, texture_y as u32, bit_depth, obj_width, data);

          let tile_address = tile_number as u32 * boundary + offset * bit_depth * 8;

          let palette_index = if obj_attributes.palette_flag {
            Self::get_obj_pixel_index_bpp8(
              tile_address,
              tile_x as u16,
              tile_y as u16,
              false,
              false,
              vram,
              is_engine_b
            )
          } else {
            Self::get_obj_pixel_index_bpp4(
              tile_address, tile_x as u16,
              tile_y as u16,
              false,
              false,
              vram,
              is_engine_b
            )
          };

          if palette_index != 0 {
            let color = if bit_depth == 8 && data.dispcnt.flags.contains(DisplayControlRegisterFlags::OBJ_EXTENDED_PALETTES) {
              Self::get_obj_extended_palette(
                palette_index as u32,
                obj_attributes.palette_number as u32,
                vram,
                is_engine_b
              )
            } else {
              Self::get_obj_palette_color(palette_index as usize, palette_bank as usize, data)
            };

            data.obj_lines[x as usize] = ObjectPixel {
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

  pub fn render_normal_line(y: u16, vram: &VRam, frame_buffer: &[Pixel3d], is_engine_b: bool, data: &mut RenderingData) {
    if data.dispcnt.flags.contains(DisplayControlRegisterFlags::DISPLAY_OBJ) {
      Self::render_objects(y, vram, is_engine_b, data);
    }

    if Self::bg_mode_enabled(0, data) {
      if !is_engine_b && (data.dispcnt.bg_mode == BgMode::Mode6 || data.dispcnt.flags.contains(DisplayControlRegisterFlags::BG_3D_SELECTION)) {
        Self::render_3d_line(y, frame_buffer, data);
      } else {
        Self::render_text_line(0, y, vram, is_engine_b, data);
      }
    }

    match data.dispcnt.bg_mode {
      BgMode::Mode0 => {
        for i in 1..4 {
          if Self::bg_mode_enabled(i, data) {
            Self::render_text_line(i, y, vram, is_engine_b, data);
          }
        }
      }
      BgMode::Mode1 => {
        for i in 1..3 {
          if Self::bg_mode_enabled(i, data) {
            Self::render_text_line(i, y, vram, is_engine_b, data);
          }
        }

        if Self::bg_mode_enabled(3, data) {
          Self::render_affine_line(3, y, vram, AffineType::Normal, is_engine_b, data);
        }
      }
      BgMode::Mode2 => {
        for i in 1..2 {
          if Self::bg_mode_enabled(i, data) {
            Self::render_text_line(i, y, vram, is_engine_b, data);
          }
        }

        for i in 2..4 {
          if Self::bg_mode_enabled(i, data) {
            Self::render_affine_line(i, y, vram, AffineType::Normal, is_engine_b, data);
          }
        }
      }
      BgMode::Mode3 => {
        for i in 1..3 {
          if Self::bg_mode_enabled(i, data) {
            Self::render_text_line(i, y, vram, is_engine_b, data);
          }
        }

        if Self::bg_mode_enabled(3, data) {
          Self::render_extended_line(3, y, vram, is_engine_b, data);
        }
      }
      BgMode::Mode4 => {
        if Self::bg_mode_enabled(1, data) {
          Self::render_text_line(1, y, vram, is_engine_b, data);
        }


        if Self::bg_mode_enabled(2, data) {
          Self::render_affine_line(2, y, vram, AffineType::Normal, is_engine_b, data);
        }

        if Self::bg_mode_enabled(3, data) {
          Self::render_extended_line(3, y, vram, is_engine_b, data);
        }
      }
      BgMode::Mode5 => {
        if Self::bg_mode_enabled(1, data) {
          Self::render_text_line(1, y, vram, is_engine_b, data);
        }

        if Self::bg_mode_enabled(2, data) {
          Self::render_extended_line(2, y, vram, is_engine_b, data);
        }

        if Self::bg_mode_enabled(3, data) {
          Self::render_extended_line(3, y, vram, is_engine_b, data);
        }
      }
      BgMode::Mode6 => {
        Self::render_affine_line(2, y, vram, AffineType::Large, is_engine_b, data)
      }
    }

    Self::finalize_scanline(y, data);
  }

  fn render_extended_line(bg_index: usize, y: u16, vram: &VRam, is_engine_b: bool, data: &mut RenderingData) {
    if data.bgcnt[bg_index].contains(BgControlRegister::PALETTES) {
      if data.bgcnt[bg_index].character_base_block() & 0b1 != 0 {
        // Extended Direct
        Self::render_affine_line(bg_index, y, vram, AffineType::Extended8bppDirect, is_engine_b, data);
      } else {
        // Extended8bpp
        Self::render_affine_line(bg_index, y, vram, AffineType::Extended8bpp, is_engine_b, data);
      }
    } else {
      // Extended
      Self::render_affine_line(bg_index, y, vram, AffineType::Extended, is_engine_b, data);
    }
  }

  fn render_3d_line(y: u16, frame_buffer: &[Pixel3d], data: &mut RenderingData) {

    for x in 0..SCREEN_WIDTH {
      let pixel = frame_buffer[(x + y * SCREEN_WIDTH) as usize];

      data.bg_lines[0][x as usize] = pixel.color;
    }
  }

  fn render_objects(y: u16, vram: &VRam, is_engine_b: bool, data: &mut RenderingData) {
    for i in 0..128 {
      let obj_attributes = Self::get_attributes(i, data);

      if obj_attributes.obj_disable {
        continue;
      }
      if obj_attributes.rotation_flag {
        Self::render_affine_object(obj_attributes, y, vram, is_engine_b, data);
      } else {
        Self::render_normal_object(obj_attributes, y, vram, is_engine_b, data);
      }
    }
  }

  fn get_obj_affine_params(affine_index: u16, data: &RenderingData) -> (i16, i16, i16, i16) {
    let mut offset = affine_index * 32 + AFFINE_SIZE;

    let dx = Self::oam_read_16(offset as usize, data) as i16;
    offset += 2 + AFFINE_SIZE;
    let dmx = Self::oam_read_16(offset as usize, data) as i16;
    offset += 2 + AFFINE_SIZE;
    let dy = Self::oam_read_16(offset as usize, data) as i16;
    offset += 2 + AFFINE_SIZE;
    let dmy = Self::oam_read_16(offset as usize, data) as i16;

    (dx, dmx, dy, dmy)
  }

  fn get_attributes(i: usize, data: &RenderingData) -> OamAttributes {
    let oam_address = i * ATTRIBUTE_SIZE;

    let attribute1 = Self::oam_read_16(oam_address, data);
    let attribute2 = Self::oam_read_16(oam_address + 2, data);
    let attribute3 = Self::oam_read_16(oam_address + 4, data);

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

  pub fn bg_mode_enabled(bg_index: usize, data: &RenderingData) -> bool {
    match bg_index {
      0 => data.dispcnt.flags.contains(DisplayControlRegisterFlags::DISPLAY_BG0),
      1 => data.dispcnt.flags.contains(DisplayControlRegisterFlags::DISPLAY_BG1),
      2 => data.dispcnt.flags.contains(DisplayControlRegisterFlags::DISPLAY_BG2),
      3 => data.dispcnt.flags.contains(DisplayControlRegisterFlags::DISPLAY_BG3),
      _ => unreachable!("can't happen")
    }
  }

  fn render_text_line(bg_index: usize, y: u16, vram: &VRam, is_engine_b: bool, data: &mut RenderingData) {
    let (x_offset, y_offset) = (data.bgxofs[bg_index], data.bgyofs[bg_index]);
    /*
      engine A screen base: BGxCNT.bits*2K + DISPCNT.bits*64K
      engine B screen base: BGxCNT.bits*2K + 0
      engine A char base: BGxCNT.bits*16K + DISPCNT.bits*64K
      engine B char base: BGxCNT.bits*16K + 0
     */
    let (tilemap_base, tile_base) = Self::get_tile_base_addresses(bg_index, is_engine_b, data);

    let mut x = 0;

    let (background_width, background_height) = data.bgcnt[bg_index].get_screen_dimensions();

    let x_in_bg = (x + x_offset) % background_width;
    let y_in_bg = (y + y_offset) % background_height;

    let mut x_tile_number = (x_in_bg as u32 / 8) % 32;

    let y_tile_number = (y_in_bg as u32 / 8) % 32;

    let mut x_pos_in_tile = x_in_bg % 8;
    let y_pos_in_tile = y_in_bg % 8;

    let mut screen_index = match data.bgcnt[bg_index].screen_size() {
      0 => 0,
      1 => x_in_bg / 256, // 512 x 256
      2 => y_in_bg / 256, // 256 x 512
      3 => (x_in_bg / 256) + ((y_in_bg / 256) * 2), // 512 x 512
      _ => unreachable!("not possible")
    };

    let is_bpp8 = data.bgcnt[bg_index].contains(BgControlRegister::PALETTES);

    let bit_depth = if is_bpp8 { 8 } else { 4 };

    while x < SCREEN_WIDTH {
      let tilemap_number = x_tile_number + y_tile_number * 32;
      let mut tilemap_address = tilemap_base + 0x800 * screen_index as u32 + 2 * tilemap_number as u32;
      'outer: for _ in x_tile_number..32 {
        let attributes = if !is_engine_b {
          vram.read_engine_a_bg_16(tilemap_address)
        } else {
          vram.read_engine_b_bg_16(tilemap_address)
        };

        let x_flip = (attributes >> 10) & 0x1 == 1;
        let y_flip =  (attributes >> 11) & 0x1 == 1;
        let palette_number = (attributes >> 12) & 0xf;
        let tile_number = attributes & 0x3ff;

        let tile_address = tile_base + tile_number as u32 * bit_depth * 8;

        for tile_x in x_pos_in_tile..8 {
          let palette_index = if bit_depth == 8 {
            Self::get_bg_pixel_index_bpp8(tile_address, tile_x, y_pos_in_tile, x_flip, y_flip, vram, is_engine_b)
          } else {
            Self::get_bg_pixel_index_bpp4(tile_address, tile_x, y_pos_in_tile, x_flip, y_flip, vram, is_engine_b)
          };

          let palette_bank = if is_bpp8 {
            0
          } else {
            palette_number
          };

          data.bg_lines[bg_index][x as usize] = if is_bpp8 && data.dispcnt.flags.contains(DisplayControlRegisterFlags::BG_EXTENDED_PALETTES) {
            Self::get_bg_extended_palette_color(bg_index, palette_index as usize, palette_number as usize, vram,  is_engine_b, data)
          } else {
            Self::get_bg_palette_color(palette_index as usize, palette_bank as usize, data)
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

  fn get_boundary_and_offset(x_pos_in_sprite: u32, y_pos_in_sprite: u32, bit_depth: u32, obj_width: u32, data: &RenderingData) -> (u32, u32) {
    if !data.dispcnt.flags.contains(DisplayControlRegisterFlags::TILE_OBJ_MAPPINGS) {
      (
        32 as u32,
        y_pos_in_sprite as u32 / 8 * 0x80 / (bit_depth as u32) + (x_pos_in_sprite  as u32) / 8,
      )
    } else {
      (
        32 << data.dispcnt.tile_obj_boundary as u32,
        (y_pos_in_sprite as u32 / 8 * obj_width + x_pos_in_sprite) / 8,
      )
    }
  }

  fn get_palette_color(index: usize, palette_bank: usize, offset: usize, data: &RenderingData) -> Option<Color> {
    let value = if index == 0 || (palette_bank != 0 && index % 16 == 0) {
      COLOR_TRANSPARENT
    } else {
      let index = 2 * index + 32 * palette_bank + offset;

      let lower = data.palette_ram[index];
      let upper = data.palette_ram[index + 1];

      (lower as u16) | (upper as u16) << 8
    };

    if value == COLOR_TRANSPARENT {
      None
    } else {
      Some(Color::from(value))
    }
  }

  fn get_bg_extended_palette_color(bg_index: usize, palette_index: usize, palette_bank: usize, vram: &VRam, is_engine_b: bool, data: &RenderingData) -> Option<Color> {
    let slot = if bg_index < 2 && data.bgcnt[bg_index].contains(BgControlRegister::DISPLAY_AREA_OVERFLOW) {
      bg_index + 2
    } else {
      bg_index
    };

    let address = slot as u32 * 8 * 0x400 + (palette_index + palette_bank * 256) as u32 * 2;

    let color_raw = if !is_engine_b {
      vram.read_engine_a_extended_bg_palette(address)
    } else {
      vram.read_engine_b_extended_bg_palette(address)
    };

    if color_raw != COLOR_TRANSPARENT && palette_index != 0 {
      Some(Color::from(color_raw))
    } else {
      None
    }
  }

  fn get_bg_palette_color(index: usize, palette_bank: usize, data: &RenderingData) -> Option<Color> {
    Self::get_palette_color(index, palette_bank, 0, data)
  }

  fn get_obj_palette_color(index: usize, palette_bank: usize, data: &RenderingData) -> Option<Color> {
    Self::get_palette_color(index, palette_bank, OBJ_PALETTE_OFFSET, data)
  }

  fn get_obj_extended_palette(index: u32, palette_bank: u32, vram: &VRam, is_engine_b: bool) -> Option<Color> {
    let address = (palette_bank * 256 + index) * 2;

    let color = if !is_engine_b {
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

  fn get_obj_pixel_index_bpp8(
    address: u32,
    tile_x: u16,
    tile_y: u16,
    x_flip: bool,
    y_flip: bool,
    vram: &VRam,
    is_engine_b: bool) -> u8
  {
    let tile_x = if x_flip { 7 - tile_x } else { tile_x };
    let tile_y = if y_flip { 7 - tile_y } else { tile_y };

    if !is_engine_b {
      vram.read_engine_a_obj(address + tile_x as u32 + (tile_y as u32) * 8)
    } else {
      vram.read_engine_b_obj(address + tile_x as u32 + (tile_y as u32) * 8)
    }
  }

  fn get_obj_pixel_index_bpp4(
    address: u32,
    tile_x: u16,
    tile_y: u16,
    x_flip: bool,
    y_flip: bool,
    vram: &VRam,
    is_engine_b: bool) -> u8
  {
    let tile_x = if x_flip { 7 - tile_x } else { tile_x };
    let tile_y = if y_flip { 7 - tile_y } else { tile_y };

    let address = address + (tile_x / 2) as u32 + (tile_y as u32) * 4;

    let byte = if !is_engine_b {
      vram.read_engine_a_obj(address)
    } else {
      vram.read_engine_b_obj(address)
    };

    if tile_x & 0b1 == 1 {
      byte >> 4
    } else {
      byte & 0xf
    }
  }

  fn get_bg_pixel_index_bpp8(address: u32, tile_x: u16, tile_y: u16, x_flip: bool, y_flip: bool, vram: &VRam, is_engine_b: bool) -> u8 {
    let tile_x = if x_flip { 7 - tile_x } else { tile_x };
    let tile_y = if y_flip { 7 - tile_y } else { tile_y };

    if !is_engine_b {
      vram.read_engine_a_bg(address + tile_x as u32 + (tile_y as u32) * 8)
    } else {
      vram.read_engine_b_bg(address + tile_x as u32 + (tile_y as u32) * 8)
    }
  }

  fn get_bg_pixel_index_bpp4(address: u32, tile_x: u16, tile_y: u16, x_flip: bool, y_flip: bool, vram: &VRam, is_engine_b: bool) -> u8 {
    let tile_x = if x_flip { 7 - tile_x } else { tile_x };
    let tile_y = if y_flip { 7 - tile_y } else { tile_y };

    let address = address + (tile_x / 2) as u32 + (tile_y as u32) * 4;

    let byte = if !is_engine_b {
      vram.read_engine_a_bg(address)
    } else {
      vram.read_engine_b_bg(address)
    };

    if tile_x & 0b1 == 1 {
      byte >> 4
    } else {
      byte & 0xf
    }
  }

  fn get_obj_coordinates(x: u16, y: u16) -> (i16, i16) {
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

  fn render_bitmap_object(
    x: usize,
    x_pos_in_sprite: u32,
    y_pos_in_sprite: u32,
    obj_width: u32,
    obj_attributes: &OamAttributes,
    vram: &VRam,
    is_engine_b: bool,
    data: &mut RenderingData)
  {
    // 1d object
    let (tile_base, width) = if data.dispcnt.flags.contains(DisplayControlRegisterFlags::BITMAP_OBJ_MAPPING) {
      // means the object is a square which isnt allowed in 1d mode
      if data.dispcnt.flags.contains(DisplayControlRegisterFlags::BITMAP_OBJ_2D_DIMENSION) {
        return;
      }
      let boundary = if data.dispcnt.flags.contains(DisplayControlRegisterFlags::BITMAP_OBJ_1D_BOUNDARY) {
        256
      } else {
        128
      };

      (obj_attributes.tile_number * boundary, obj_width)
    } else {
      let mut mask_x = 0xf;
      let mut width = 128;

      if data.dispcnt.flags.contains(DisplayControlRegisterFlags::BITMAP_OBJ_2D_DIMENSION) {
        mask_x = 0x1f;
        width = 256;
      }

      // 2D_BitmapVramAddress = (TileNo AND MaskX)*10h + (TileNo AND NOT MaskX)*80h
      ((obj_attributes.tile_number & mask_x) * 0x10 + (obj_attributes.tile_number & !mask_x) * 0x80, width)
    };

    let tile_address = tile_base as u32 + 2 * (x_pos_in_sprite + y_pos_in_sprite * width);

    let color_raw = if !is_engine_b {
      vram.read_engine_a_obj_16(tile_address)
    } else {
      vram.read_engine_b_obj_16(tile_address)
    };

    let color = if color_raw == 0 {
      None
    } else {
      Some(Color::from(color_raw))
    };

    data.obj_lines[x] = ObjectPixel {
      priority: obj_attributes.priority,
      color,
      is_window: obj_attributes.obj_mode == 2,
      is_transparent: obj_attributes.obj_mode == 1
    };
  }

  fn render_normal_object(obj_attributes: OamAttributes, y: u16, vram: &VRam, is_engine_b: bool, data: &mut RenderingData) {
    let (obj_width, obj_height) = obj_attributes.get_object_dimensions();

    let (x_coordinate, y_coordinate) = Self::get_obj_coordinates(obj_attributes.x_coordinate, obj_attributes.y_coordinate);

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

      if data.obj_lines[screen_x as usize].priority <= obj_attributes.priority && obj_attributes.obj_mode != 2 {
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
        Self::render_bitmap_object(
          screen_x as usize,
          x_pos_in_sprite,
          y_pos_in_sprite as u32,
          obj_width,
          &obj_attributes,
          vram,
          is_engine_b,
          data
        );
      } else {
        let x_pos_in_tile = x_pos_in_sprite % 8;
        let y_pos_in_tile = y_pos_in_sprite % 8;

        let (boundary, offset) = Self::get_boundary_and_offset(x_pos_in_sprite as u32, y_pos_in_sprite as u32, bit_depth, obj_width, data);

        let tile_address = tile_number as u32 * boundary + offset * bit_depth * 8;

        let palette_index = if bit_depth == 8 {
          Self::get_obj_pixel_index_bpp8(
            tile_address,
            x_pos_in_tile as u16,
            y_pos_in_tile,
            false,
            false,
            &vram,
            is_engine_b
          )
        } else {
          Self::get_obj_pixel_index_bpp4(
            tile_address,
            x_pos_in_tile as u16,
            y_pos_in_tile,
            false,
            false,
            &vram,
            is_engine_b
          )
        };


        if palette_index != 0 {
          // need to determine whether to look at extended palette or regular obj palette
          let color = if bit_depth == 8 && data.dispcnt.flags.contains(DisplayControlRegisterFlags::OBJ_EXTENDED_PALETTES) {
            Self::get_obj_extended_palette(palette_index as u32, obj_attributes.palette_number as u32, vram, is_engine_b)
          } else {
            if bit_depth == 8 {
              palette_bank = 0;
            }
            Self::get_obj_palette_color(palette_index as usize, palette_bank as usize, data)
          };
          data.obj_lines[screen_x as usize] = ObjectPixel {
            priority: obj_attributes.priority,
            color,
            is_window: obj_attributes.obj_mode == 2,
            is_transparent: obj_attributes.obj_mode == 1
          };
        }
      }
    }
  }

  pub fn render_line(&mut self, y: u16, vram: &mut VRam, frame_buffer: &[Pixel3d; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize], is_engine_b: bool) {
    let data = &mut self.thread_data.rendering_data[!is_engine_b as usize].lock().unwrap();
    match data.dispcnt.display_mode {
      DisplayMode::Mode0 => {
        let color = Color {
          r: 0xff,
          g: 0xff,
          b: 0xff,
          alpha: None
        };

        for x in 0..SCREEN_WIDTH {
          Self::set_pixel(x as usize, y as usize, color, data);
        }
      },
      DisplayMode::Mode1 => Self::render_normal_line(y, vram, frame_buffer, is_engine_b, data),
      DisplayMode::Mode2 => {
        for x in 0..SCREEN_WIDTH {
          let index = 2 * (y as usize * SCREEN_WIDTH as usize + x as usize);
          let bank = vram.get_lcdc_bank(data.dispcnt.vram_block);

          let color = bank[index] as u16 | (bank[(index + 1) as usize] as u16) << 8;

          let color = Color::to_rgb24(color);

          Self::set_pixel(x as usize, y as usize, color, data);
        }
      }
      DisplayMode::Mode3 => todo!()
    }
    drop(data);
  }

  fn oam_read_16(address: usize, data: &RenderingData) -> u16 {
    (data.oam[address] as u16) | (data.oam[address + 1] as u16) << 8
  }

  pub fn set_pixel(x: usize, y: usize, color: Color, data: &mut RenderingData) {
    let i: usize = 3 * (x + y * SCREEN_WIDTH as usize);

    data.pixels[i] = color.r;
    data.pixels[i + 1] = color.g;
    data.pixels[i + 2] = color.b;
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

  fn get_tile_base_addresses(bg_index: usize, is_engine_b: bool, data: &RenderingData) -> (u32, u32) {
    if !is_engine_b {
      (data.bgcnt[bg_index].screen_base_block() as u32 * 0x800 + data.dispcnt.screen_base * 0x1_0000, data.bgcnt[bg_index].character_base_block() as u32 * 0x4000 + data.dispcnt.character_base * 0x1_0000)
    } else {
      (data.bgcnt[bg_index].screen_base_block() as u32 * 0x800, data.bgcnt[bg_index].character_base_block() as u32 * 0x4000)
    }
  }

  fn render_affine_line(bg_index: usize, y: u16, vram: &VRam, affine_type: AffineType, is_engine_b: bool, data: &mut RenderingData) {
    let (dx, dy) = (data.bg_props[bg_index - 2].dx, data.bg_props[bg_index - 2].dy);

    let (tilemap_base, tile_base) = Self::get_tile_base_addresses(bg_index, is_engine_b, data);

    let texture_size = if affine_type != AffineType::Large {
      128 << data.bgcnt[bg_index].screen_size()
    } else {
      512 << (data.bgcnt[bg_index].screen_size() & 0b1)
    };

    let (mut ref_x, mut ref_y) = (data.bg_props[bg_index - 2].internal_x, data.bg_props[bg_index - 2].internal_y);

    data.bg_props[bg_index - 2].internal_x += data.bg_props[bg_index - 2].dmx as i32;
    data.bg_props[bg_index - 2].internal_y += data.bg_props[bg_index - 2].dmy as i32;

    for x in 0..SCREEN_WIDTH {
      let mut transformed_x = ref_x >> 8;
      let mut transformed_y = ref_y >> 8;

      ref_x += dx as i32;
      ref_y += dy as i32;

      if transformed_x < 0 || transformed_x >= texture_size || transformed_y < 0 || transformed_y >= texture_size {
        if data.bgcnt[bg_index].contains(BgControlRegister::DISPLAY_AREA_OVERFLOW) {
          transformed_x = transformed_x % texture_size;
          transformed_y = transformed_y % texture_size;
        } else {
          data.bg_lines[bg_index][x as usize] = None;
          continue;
        }
      }

      let x_pos_in_tile = transformed_x % 8;
      let y_pos_in_tile = transformed_y % 8;

      // formulas for extended lines:
      // for extended 8bpp direct color = 2*(transformed_y * texture_size + x);
      // for extended 8bpp, palette_index = transformed_y * WIDTH + x,
      // for extended, get the attributes from vram and render accordingly
      data.bg_lines[bg_index][x as usize] = match affine_type {
        AffineType::Extended => {
          let bit_depth = 8;

          let tilemap_address = Self::get_extended_tilemap_address(tilemap_base, transformed_x, transformed_y, texture_size);

          let attributes = if !is_engine_b {
            vram.read_engine_a_bg_16(tilemap_address)
          } else {
            vram.read_engine_b_bg_16(tilemap_address)
          };

          let x_flip = (attributes >> 10) & 0x1 == 1;
          let y_flip =  (attributes >> 11) & 0x1 == 1;
          let palette_number = (attributes >> 12) & 0xf;
          let tile_number = attributes & 0x3ff;

          let tile_address = tile_base + tile_number as u32 * bit_depth * 8;

          let palette_index = Self::get_bg_pixel_index_bpp8(tile_address, x_pos_in_tile as u16, y_pos_in_tile as u16, x_flip, y_flip, vram, is_engine_b);

          if data.dispcnt.flags.contains(DisplayControlRegisterFlags::BG_EXTENDED_PALETTES) {
            Self::get_bg_extended_palette_color(bg_index, palette_index as usize, palette_number as usize, vram, is_engine_b, data)
          } else {
            Self::get_bg_palette_color(palette_index as usize, 0, data)
          }
        }
        AffineType::Normal => {
          let bit_depth = 8;

          let tilemap_address = Self::get_affine_tilemap_address(tilemap_base, transformed_x, transformed_y, texture_size);

          let tile_number = if !is_engine_b {
            vram.read_engine_a_bg(tilemap_address)
          } else {
            vram.read_engine_b_bg(tilemap_address)
          };

          let tile_address = tile_base + tile_number as u32 * bit_depth as u32 * 8;

          let palette_index = Self::get_bg_pixel_index_bpp8(
            tile_address,
            x_pos_in_tile as u16,
            y_pos_in_tile as u16,
            false,
            false,
            vram,
            is_engine_b
          );

          Self::get_bg_palette_color(palette_index as usize, 0, data)
        }
        AffineType::Extended8bppDirect => {
          let address = 2 * (transformed_y as u32 * texture_size as u32 + x as u32);
          let color_raw = if !is_engine_b {
            vram.read_engine_a_bg_16(address)
          } else {
            vram.read_engine_b_bg_16(address)
          };

          if color_raw == 0 {
            None
          } else {
            Some(Color::from(color_raw))
          }
        }
        AffineType::Extended8bpp => {
          let palette_address = transformed_y as u32 * SCREEN_WIDTH as u32 + x as u32;

          let palette_index = if !is_engine_b {
            vram.read_engine_a_bg(palette_address)
          } else {
            vram.read_engine_b_bg(palette_address)
          };

          Self::get_bg_palette_color(palette_index as usize, 0, data)
        }
        AffineType::Large => {
          let palette_address = transformed_y as u32 * texture_size as u32 + transformed_x as u32;

          let palette_index = if !is_engine_b {
            vram.read_engine_a_bg(palette_address)
          } else {
            vram.read_engine_b_bg(palette_address)
          };

          Self::get_bg_palette_color(palette_index as usize, 0, data)
        }
      };
    }
  }
}