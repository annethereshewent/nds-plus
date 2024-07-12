use super::{
  registers::{
    alpha_blend_register::AlphaBlendRegister,
    bg_control_register::BgControlRegister,
    brightness_register::BrightnessRegister,
    color_effects_register::ColorEffectsRegister,
    display_control_register::{
      BgMode, DisplayControlRegister,
      DisplayControlRegisterFlags, DisplayMode
    },
    master_brightness_register::MasterBrightnessRegister,
    window_horizontal_register::WindowHorizontalRegister,
    window_in_register::WindowInRegister,
    window_out_register::WindowOutRegister,
    window_vertical_register::WindowVerticalRegister
  },
  vram::VRam,
  BgProps,
  SCREEN_HEIGHT,
  SCREEN_WIDTH
};

const COLOR_TRANSPARENT: u16 = 0x8000;
const ATTRIBUTE_SIZE: usize = 8;

#[derive(Debug)]
struct OamAttributes {
  x_coordinate: u16,
  y_coordinate: u16,
  rotation_flag: bool,
  double_sized_flag: bool,
  obj_disable: bool,
  obj_mode: u16,
  obj_mosaic: bool,
  palette_flag: bool,
  obj_shape: u16,
  obj_size: u16,
  rotation_param_selection: u16,
  horizontal_flip: bool,
  vertical_flip: bool,
  tile_number: u16,
  priority: u16,
  palette_number: u16
}

impl OamAttributes {
  pub fn get_object_dimensions(&self) -> (u32, u32) {
    match (self.obj_size, self.obj_shape) {
      (0, 0) => (8, 8),
      (1, 0) => (16, 16),
      (2, 0) => (32, 32),
      (3, 0) => (64, 64),
      (0, 1) => (16, 8),
      (1, 1) => (32, 8),
      (2, 1) => (32, 16),
      (3, 1) => (64, 32),
      (0, 2) => (8, 16),
      (1, 2) => (8, 32),
      (2, 2) => (16, 32),
      (3, 2) => (32, 64),
      _ => (8, 8)
    }
  }
}

#[derive(Copy, Clone)]
pub struct ObjectPixel {
  pub priority: u16,
  pub color: Option<Color>,
  pub is_window: bool,
  pub is_transparent: bool
}

impl ObjectPixel {
  pub fn new() -> Self {
    Self {
      priority: 4,
      color: None,
      is_window: false,
      is_transparent: false
    }
  }
}

#[derive(Copy, Clone, Debug)]
pub struct Color {
  pub r: u8,
  pub g: u8,
  pub b: u8
}

impl Color {
  pub fn from(val: u16) -> Self {
    let mut r = (val & 0x1f) as u8;
    let mut g = ((val >> 5) & 0x1f) as u8;
    let mut b = ((val >> 10) & 0x1f) as u8;

    r = (r << 3) | (r >> 2);
    g = (g << 3) | (g >> 2);
    b = (b << 3) | (b >> 2);

    Self {
      r,
      g,
      b
    }
  }

  pub fn into_rgb15(&self) -> u16 {
    self.r as u16 | (self.g as u16) << 5 |  (self.b as u16) << 5
  }
}

pub struct Engine2d<const IS_ENGINE_B: bool> {
  pub dispcnt: DisplayControlRegister<IS_ENGINE_B>,
  pub oam: [u8; 0x400],
  pub pixels: [u8; 3 * (SCREEN_WIDTH * SCREEN_HEIGHT) as usize],
  pub winin: WindowInRegister,
  pub winout: WindowOutRegister,
  pub winh: [WindowHorizontalRegister; 2],
  pub winv: [WindowVerticalRegister; 2],
  pub bldcnt: ColorEffectsRegister,
  pub bldalpha: AlphaBlendRegister,
  pub bldy: BrightnessRegister,
  pub bgcnt: [BgControlRegister; 4],
  pub bgxofs: [u16; 4],
  pub bgyofs: [u16; 4],
  pub bg_props: [BgProps; 2],
  bg_lines: [[Option<Color>; SCREEN_WIDTH as usize]; 4],
  obj_lines: Box<[ObjectPixel]>,
  pub master_brightness: MasterBrightnessRegister,
  pub bg_palette_ram: [u8; 0x200],
  pub obj_palette_ram: [u8; 0x200]
}

impl<const IS_ENGINE_B: bool> Engine2d<IS_ENGINE_B> {
  pub fn new() -> Self {
    Self {
      dispcnt: DisplayControlRegister::new(),
      oam: [0; 0x400],
      pixels: [0; 3 * (SCREEN_WIDTH * SCREEN_HEIGHT) as usize],
      bgxofs: [0; 4],
      bgyofs: [0; 4],
      bg_props: [BgProps::new(); 2],
      winh: [WindowHorizontalRegister::new(); 2],
      winv: [WindowVerticalRegister::new(); 2],
      winin: WindowInRegister::from_bits_retain(0),
      winout: WindowOutRegister::from_bits_retain(0),
      bldcnt: ColorEffectsRegister::new(),
      bldalpha: AlphaBlendRegister::new(),
      bldy: BrightnessRegister::new(),
      bgcnt: [BgControlRegister::from_bits_retain(0); 4],
      bg_lines: [[None; SCREEN_WIDTH as usize]; 4],
      master_brightness: MasterBrightnessRegister::new(),
      bg_palette_ram: [0; 0x200],
      obj_palette_ram: [0; 0x200],
      obj_lines: vec![ObjectPixel::new(); (SCREEN_WIDTH * SCREEN_HEIGHT) as usize].into_boxed_slice()
    }
  }

  pub fn write_palette_ram(&mut self, address: u32, byte: u8) {
    let mut address = address & (2 * self.bg_palette_ram.len() - 1) as u32;

    let ram = if address < self.bg_palette_ram.len() as u32 {
      &mut self.bg_palette_ram
    } else {
      &mut self.obj_palette_ram
    };

    address = address & (ram.len() - 1) as u32;

    ram[address as usize] = byte;
  }

  pub fn render_normal_line(&mut self, y: u16, vram: &VRam) {
    if self.dispcnt.flags.contains(DisplayControlRegisterFlags::DISPLAY_OBJ) {
      self.render_objects(y, vram);
    }

    match self.dispcnt.bg_mode {
      BgMode::Mode0 => {
        for i in 0..4 {
          if self.bg_mode_enabled(i) {
            self.render_text_line(i, y, vram);
          }
        }
      }
      BgMode::Mode1 => {
        for i in 0..3 {
          if self.bg_mode_enabled(i) {
            self.render_text_line(i, y, vram);
          }
        }

        if self.bg_mode_enabled(3) {
          self.render_affine_line(3);
        }
      }
      BgMode::Mode2 => {
        for i in 0..2 {
          if self.bg_mode_enabled(i) {
            self.render_text_line(i, y, vram);
          }
        }

        for i in 2..4 {
          if self.bg_mode_enabled(i) {
            self.render_affine_line(i);
          }
        }
      }
      BgMode::Mode3 => {
        for i in 0..3 {
          if self.bg_mode_enabled(i) {
            self.render_text_line(i, y, vram);
          }
        }

        if self.bg_mode_enabled(3) {
          self.render_extended_line(3);
        }
      }
      BgMode::Mode4 => {
        for i in 0..2 {
          if self.bg_mode_enabled(i) {
            self.render_text_line(i, y, vram);
          }
        }

        if self.bg_mode_enabled(2) {
          self.render_affine_line(2);
        }

        if self.bg_mode_enabled(3) {
          self.render_extended_line(3);
        }
      }
      BgMode::Mode5 => {
        for i in 0..2 {
          if self.bg_mode_enabled(i) {
            self.render_text_line(i, y, vram);
          }
        }

        if self.bg_mode_enabled(2) {
          self.render_extended_line(2);
        }

        if self.bg_mode_enabled(3) {
          self.render_extended_line(3);
        }
      }
      BgMode::Mode6 => (), // TODO
      _ => panic!("reserved option given for bg mode: 7")
    }

    self.finalize_scanline(y);
  }

  fn render_extended_line(&mut self, bg_index: usize) {

  }

  fn finalize_scanline(&mut self, y: u16) {
    for x in 0..SCREEN_WIDTH {
      for i in 0..4 {
        if self.bg_mode_enabled(i) {
          if let Some(color) = self.bg_lines[i][x as usize] {
            self.set_pixel(x as usize, y as usize, color);
            break;
          }
        }
      }
      // render objects
      let obj_index = x + y * SCREEN_WIDTH;
      if let Some(color) = self.obj_lines[obj_index as usize].color {
        self.set_pixel(x as usize, y as usize, color);
      }
    }
  }

  pub fn clear_obj_lines(&mut self) {
    for x in &mut self.obj_lines.iter_mut() {
      *x = ObjectPixel::new();
    }
  }

  fn render_affine_line(&mut self, bg_index: usize) {

  }
  fn render_objects(&mut self, y: u16, vram: &VRam) {
    for i in 0..128 {
      let obj_attributes = self.get_attributes(i);

      if obj_attributes.obj_disable {
        continue;
      }
      if obj_attributes.rotation_flag {
        // self.render_affine_object(obj_attributes);
      } else {
        // render object normally
        self.render_normal_object(obj_attributes, y, vram);
      }
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

  fn render_normal_object(&mut self, obj_attributes: OamAttributes, y: u16, vram: &VRam) {
    let (obj_width, obj_height) = obj_attributes.get_object_dimensions();

    let (x_coordinate, y_coordinate) = self.get_obj_coordinates(obj_attributes.x_coordinate, obj_attributes.y_coordinate);

    let y_pos_in_sprite: i16 = y as i16 - y_coordinate;

    if y_pos_in_sprite < 0 || y_pos_in_sprite as u32 >= obj_height || obj_attributes.obj_mode == 3 {
      return;
    }

    let tile_number = obj_attributes.tile_number;

    let bit_depth = if obj_attributes.palette_flag {
      8
    } else {
      4
    };

    let tile_width = if self.dispcnt.flags.contains(DisplayControlRegisterFlags::BITMAP_OBJ_MAPPING) {
      obj_width / 8
    } else {
      if obj_attributes.palette_flag {
        16
      } else {
        32
      }
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

      let obj_line_index = (screen_x as u16 + y * SCREEN_WIDTH) as usize;

      if self.obj_lines[obj_line_index].priority <= obj_attributes.priority && obj_attributes.obj_mode != 2 {
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

      let x_pos_in_tile = x_pos_in_sprite % 8;
      let y_pos_in_tile = y_pos_in_sprite % 8;

      // let tile_address = tile_base as u32 + (x_pos_in_sprite / 8  + (y_pos_in_sprite as u32 / 8) * tile_width) * tile_size;
      let (boundary, offset) = if !self.dispcnt.flags.contains(DisplayControlRegisterFlags::TILE_OBJ_MAPPINGS) {
        (
          32 as u32,
          y_pos_in_sprite as u32 / 8 * 0x80 / (bit_depth as u32) + (x_pos_in_sprite  as u32) / 8,
        )
      } else {
        (
          32 << self.dispcnt.tile_obj_boundary as u32,
          (y_pos_in_sprite as u32 / 8 * tile_width + x_pos_in_sprite) / 8,
        )
      };

      let tile_address = tile_number as u32 * boundary + offset * bit_depth * 8;

      // println!("tile address = {:x}, boundary = {:x} for coordinates {screen_x},{y}", tile_address, boundary);

      let palette_index = if bit_depth == 8 {
        self.get_obj_pixel_index_bpp8(tile_address, x_pos_in_tile as u16, y_pos_in_tile, false, false, &vram)
      } else {
        self.get_obj_pixel_index_bpp4(tile_address, x_pos_in_tile as u16, y_pos_in_tile, false, false, &vram)
      };


      if palette_index != 0 {
        // need to determine whether to look at extended palette or regular obj palette
        let color = if bit_depth == 8 && self.dispcnt.flags.contains(DisplayControlRegisterFlags::OBJ_EXTENDED_PALETTES) {
          self.get_obj_extended_palette(palette_index as u32, palette_bank as u32, vram)
        } else {
          if bit_depth == 8 {
            palette_bank = 0;
          }
          self.get_obj_palette_color(palette_index as usize, palette_bank as usize)
        };
        self.obj_lines[obj_line_index] = ObjectPixel {
          priority: obj_attributes.priority,
          color,
          is_window: obj_attributes.obj_mode == 2,
          is_transparent: obj_attributes.obj_mode == 1
        };
      }
    }
  }

  fn oam_read_16(&self, address: usize) -> u16 {
    (self.oam[address] as u16) | (self.oam[address + 1] as u16) << 8
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
    let obj_mode = (attribute1 >> 10) & 0b11;
    let obj_mosaic = (attribute1 >> 12) & 0b1 == 1;
    let palette_flag = (attribute1 >> 13) & 0b1 == 1;
    let obj_shape = (attribute1 >> 14) & 0b11;

    let x_coordinate = attribute2 & 0x1ff;
    let rotation_param_selection = if rotation_flag {
      (attribute2 >> 9) & 0b11111
    } else {
      0
    };
    let horizontal_flip = !rotation_flag && (attribute2 >> 12) & 0b1 == 1;
    let vertical_flip = !rotation_flag && (attribute2 >> 13) & 0b1 == 1;
    let obj_size = (attribute2 >> 14) & 0b11;

    let tile_number = attribute3 & 0b1111111111;
    let priority = (attribute3 >> 10) & 0b11;
    let palette_number = (attribute3 >> 12) & 0xf;

    OamAttributes {
      y_coordinate,
      rotation_flag,
      double_sized_flag,
      obj_disable,
      obj_mode,
      obj_mosaic,
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

  fn bg_mode_enabled(&self, bg_index: usize) -> bool {
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
    let (tilemap_base, tile_base) = if !IS_ENGINE_B {
      (self.bgcnt[bg_index].screen_base_block() as u32 * 0x800 + self.dispcnt.screen_base * 0x1_0000, self.bgcnt[bg_index].character_base_block() as u32 * 0x4000 + self.dispcnt.character_base * 0x1_0000)
    } else {
      (self.bgcnt[bg_index].screen_base_block() as u32 * 0x800, self.bgcnt[bg_index].character_base_block() as u32 * 0x4000)
    };

    // println!("tilemap_base = {:x} tile_base = {:x}", tilemap_base, tile_base);

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

    // let tile_size: u32 = if self.bgcnt[bg_index].contains(BgControlRegister::PALETTES) {
    //   64
    // } else {
    //   32
    // };

    let is_bpp8 = self.bgcnt[bg_index].contains(BgControlRegister::PALETTES);

    let bit_depth = if is_bpp8 { 8 } else { 4 };

    while x < SCREEN_WIDTH {
      let tile_number = x_tile_number + y_tile_number * 32;
      let mut tilemap_address = tilemap_base + 0x800 * screen_index as u32 + 2 * tile_number as u32;
      'outer: for _ in x_tile_number..32 {
        let attributes = if !IS_ENGINE_B {
          vram.read_engine_a_bg(tilemap_address) as u16 | (vram.read_engine_a_bg(tilemap_address + 1) as u16) << 8
        } else {
          vram.read_engine_b_bg(tilemap_address) as u16 | (vram.read_engine_b_bg(tilemap_address + 1) as u16) << 8
        };

        let x_flip = (attributes >> 10) & 0x1 == 1;
        let y_flip =  (attributes >> 11) & 0x1 == 1;
        let palette_number = (attributes >> 12) & 0xf;
        let tile_number = attributes & 0x3ff;

        let tile_address = tile_base + tile_number as u32 * bit_depth * 8;

        // println!("got tile_address {:x}", tile_address);

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

          self.bg_lines[bg_index][x as usize] = self.get_bg_palette_color(palette_index as usize, palette_bank as usize);

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

  pub fn render_line(&mut self, y: u16, vram: &mut VRam) {
    match self.dispcnt.display_mode {
      DisplayMode::Mode0 => {
        let color = Color {
          r: 0xff,
          g: 0xff,
          b: 0xff
        };

        for x in 0..SCREEN_WIDTH {
          self.set_pixel(x as usize, y as usize, color);
        }
      },
      DisplayMode::Mode1 => self.render_normal_line(y, vram),
      DisplayMode::Mode2 => {
        for x in 0..SCREEN_WIDTH {
          let index = 2 * (y as usize * SCREEN_WIDTH as usize + x as usize);
          let bank = vram.get_lcdc_bank(self.dispcnt.vram_block);

          let color = bank[index] as u16 | (bank[(index + 1) as usize] as u16) << 8;

          let color = Color::from(color);

          self.set_pixel(x as usize, y as usize, color);
        }
      }
      DisplayMode::Mode3 => todo!()
    }
  }

  pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
    let i: usize = 3 * (x + y * SCREEN_WIDTH as usize);

    self.pixels[i] = color.r;
    self.pixels[i + 1] = color.g;
    self.pixels[i + 2] = color.b;

  }

  pub fn read_register(&self, address: u32) -> u16 {
    match address & 0xff {
      0x08 => self.bgcnt[0].bits(),
      0x0a => self.bgcnt[1].bits(),
      0x0c => self.bgcnt[2].bits(),
      0x0e => self.bgcnt[3].bits(),
      0x40 => self.winh[0].x1,
      0x42 => self.winh[1].x1,
      0x44 => self.winv[0].y1,
      0x46 => self.winv[1].y1,
      0x48 => self.winin.bits(),
      0x4a => self.winout.bits(),
      0x4c => 0, // TODO, see below
      0x50 => self.bldcnt.value,
      0x52 => self.bldalpha.read(),
      0x54 => self.bldy.read(),
      0x56..=0x5f => 0,
      _ => panic!("invalid address given to engine read register method")
    }
  }

  pub fn write_register(&mut self, address: u32, val: u16, mask: Option<u16>) {
    let mut value = 0;

    if let Some(mask) = mask {
      value = self.read_register(address) & mask;
    }

    value |= val;

    let bg_props = &mut self.bg_props;

    macro_rules! write_bg_reference_point {
      (low $coordinate:ident $internal:ident $i:expr) => {{
        let existing = bg_props[$i].$coordinate as u32;

        let new_value = ((existing & 0xffff0000) + (value as u32)) as i32;

        bg_props[$i].$coordinate = new_value;
        bg_props[$i].$internal = new_value;
      }};
      (high $coordinate:ident $internal:ident $i:expr) => {{
        let existing = bg_props[$i].$coordinate;

        let new_value = existing & 0xffff | (((value & 0xfff) as i32) << 20) >> 4;

        bg_props[$i].$coordinate = new_value;
        bg_props[$i].$internal = new_value;
      }}
    }

    match address & 0xff {
      0x08 => self.bgcnt[0] = BgControlRegister::from_bits_retain(value),
      0x0a => self.bgcnt[1] = BgControlRegister::from_bits_retain(value),
      0x0c => self.bgcnt[2] = BgControlRegister::from_bits_retain(value),
      0x0e => self.bgcnt[3] = BgControlRegister::from_bits_retain(value),
      0x10 => self.bgxofs[0] = value & 0b111111111,
      0x12 => self.bgyofs[0] = value & 0b111111111,
      0x14 => self.bgxofs[1] = value & 0b111111111,
      0x16 => self.bgyofs[1] = value & 0b111111111,
      0x18 => self.bgxofs[2] = value & 0b111111111,
      0x1a => self.bgyofs[2] = value & 0b111111111,
      0x1c => self.bgxofs[3] = value & 0b111111111,
      0x1e => self.bgyofs[3] = value & 0b111111111,
      0x20 => self.bg_props[0].dx = value as i16,
      0x22 => self.bg_props[0].dmx = value as i16,
      0x24 => self.bg_props[0].dy = value as i16,
      0x26 => self.bg_props[0].dmy = value as i16,
      0x28 => write_bg_reference_point!(low x internal_x 0),
      0x2a => write_bg_reference_point!(high x internal_x 0),
      0x2c => write_bg_reference_point!(low y internal_y 0),
      0x2e => write_bg_reference_point!(high y internal_y 0),
      0x30 => self.bg_props[1].dx = value as i16,
      0x32 => self.bg_props[1].dmx = value as i16,
      0x34 => self.bg_props[1].dy = value as i16,
      0x36 => self.bg_props[1].dmy = value as i16,
      0x38 => write_bg_reference_point!(low x internal_x 1),
      0x3a => write_bg_reference_point!(high x internal_x 1),
      0x3c => write_bg_reference_point!(low y internal_y 1),
      0x3e => write_bg_reference_point!(high y internal_y 1),
      0x40 => self.winh[0].write(value),
      0x42 => self.winh[1].write(value),
      0x44 => self.winv[0].write(value),
      0x46 => self.winv[1].write(value),
      0x48 => self.winin = WindowInRegister::from_bits_retain(value),
      0x4a => self.winout = WindowOutRegister::from_bits_retain(value),
      0x4c..=0x4e => (), // TODO (but probably not lmao, mosaic is pointless),
      0x50 => self.bldcnt.write(value),
      0x52 => self.bldalpha.write(value),
      0x54 => self.bldy.write(value),
      0x56..=0x5f => (),
      _ => panic!("invalid address given to engine write register method")
    }
  }

  fn get_palette_color(index: usize, palette_bank: usize, ram: &[u8]) -> Option<Color> {

    let value = if index == 0 || (palette_bank != 0 && index % 16 == 0) {
      COLOR_TRANSPARENT
    } else {
      let index = 2 * index + 32 * palette_bank;

      let lower = ram[index];
      let upper = ram[index + 1];

      ((lower as u16) | (upper as u16) << 8) & 0x7fff
    };

    if value == COLOR_TRANSPARENT {
      None
    } else {
      Some(Color::from(value))
    }
  }

  fn get_bg_palette_color(&self, index: usize, palette_bank: usize) -> Option<Color> {
    Self::get_palette_color(index, palette_bank, &self.bg_palette_ram)
  }

  fn get_obj_palette_color(&self, index: usize, palette_bank: usize) -> Option<Color> {
    let address = (palette_bank * 16 + index) * 2;

    Some(Color::from(self.obj_palette_ram[address] as u16 | (self.obj_palette_ram[address + 1] as u16) << 8))
  }

  fn get_obj_extended_palette(&self, index: u32, palette_bank: u32, vram: &VRam) -> Option<Color> {
    let address = (palette_bank * 256 + index) * 2;

    let color = if !IS_ENGINE_B {
      (vram.read_engine_a_extended_palette(address) as u16) | (vram.read_engine_a_extended_palette(address + 1) as u16) << 8
    } else {
      (vram.read_engine_b_extended_palette(address) as u16) | (vram.read_engine_b_extended_palette(address + 1) as u16) << 8
    };

    Some(Color::from(color))
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

  fn get_bg_pixel_index_bpp8(&self, address: u32, tile_x: u16, tile_y: u16, x_flip: bool, y_flip: bool, vram: &VRam) -> u8 {
    let tile_x = if x_flip { 7 - tile_x } else { tile_x };
    let tile_y = if y_flip { 7 - tile_y } else { tile_y };

    // println!("reading from vram at address {:x}", address + tile_x as u32 + (tile_y as u32) * 8);

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

    // println!("reading from vram at address {:x}", address);

    let byte = if !IS_ENGINE_B {
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
}