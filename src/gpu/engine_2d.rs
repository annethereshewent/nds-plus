use super::{
  registers::{
    alpha_blend_register::AlphaBlendRegister,
    bg_control_register::BgControlRegister,
    brightness_register::BrightnessRegister,
    color_effects_register::ColorEffectsRegister,
    display_control_register::DisplayControlRegister,
    master_brightness_register::MasterBrightnessRegister,
    window_horizontal_register::WindowHorizontalRegister,
    window_in_register::WindowInRegister,
    window_out_register::WindowOutRegister,
    window_vertical_register::WindowVerticalRegister
  },
  BgProps,
  SCREEN_HEIGHT,
  SCREEN_WIDTH
};

pub mod rendering2d;
pub mod pixel_processing;

const COLOR_TRANSPARENT: u16 = 0x8000;
const ATTRIBUTE_SIZE: usize = 8;
const AFFINE_SIZE: u16 = 3 * 2;
const OBJ_PALETTE_OFFSET: usize = 0x200;

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
  pub fn convert(&mut self) -> Self {
    self.r = (self.r << 3) | (self.r >> 2);
    self.g = (self.g << 3) | (self.g >> 2);
    self.b = (self.b << 3) | (self.b >> 2);

    *self
  }

  pub fn to_rgb24(val: u16) -> Self {
    let mut r = (val & 0x1f) as u8;
    let mut g = ((val >> 5) & 0x1f) as u8;
    let mut b = ((val >> 10) & 0x1f) as u8;

    r = (r << 3) | (r >> 2);
    g = (g << 3) | (g >> 2);
    b = (b << 3) | (b >> 2);

    Color {
      r,
      g,
      b
    }
  }

  pub fn from(val: u16) -> Self {
    Color {
      r: (val & 0x1f) as u8,
      g: ((val >> 5) & 0x1f) as u8,
      b: ((val >> 10) & 0x1f) as u8
    }
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
  pub palette_ram: [u8; 0x400],
  pub debug_on: bool
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
      palette_ram: [0; 0x400],
      obj_lines: vec![ObjectPixel::new(); SCREEN_WIDTH as usize].into_boxed_slice(),
      debug_on: false
    }
  }

  pub fn write_palette_ram(&mut self, address: u32, byte: u8) {
    let index = (address as usize) & (self.palette_ram.len() - 1);

    self.palette_ram[index as usize] = byte;
  }

  pub fn clear_obj_lines(&mut self) {
    for x in &mut self.obj_lines.iter_mut() {
      *x = ObjectPixel::new();
    }
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

        let new_value = ((existing & 0xffff0000) | (value as u32)) as i32;

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
}