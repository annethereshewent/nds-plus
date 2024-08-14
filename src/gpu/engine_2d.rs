use std::mem::size_of;

use crate::number::Number;

use super::{
  color::Color, registers::{
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
  }, BgProps, SCREEN_HEIGHT, SCREEN_WIDTH
};

pub mod rendering2d;
pub mod renderer2d;
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
  _obj_mosaic: bool,
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

pub struct Engine2d<const IS_ENGINE_B: bool> {
  pub dispcnt: DisplayControlRegister,
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
  obj_lines: [ObjectPixel; SCREEN_WIDTH as usize],
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
      obj_lines: [ObjectPixel::new(); SCREEN_WIDTH as usize],
      debug_on: false
    }
  }

  pub fn write_palette_ram<T: Number>(&mut self, address: u32, byte: T) {
    let index = (address as usize) & (self.palette_ram.len() - 1);

    unsafe { *(&mut self.palette_ram[index as usize] as *mut u8 as *mut T) = byte };
  }

  pub fn read_palette_ram<T: Number>(&self, address: u32) -> T {
    let index = (address as usize) & (self.palette_ram.len() - 1);

    // unsafe { *(&self.palette_ram[index as usize] as *const u8 as *const T) }

    let mut value: T = num::zero();

    for i in 0..size_of::<T>() {
      value = num::cast::<u8, T>(self.palette_ram[(index + i) as usize] << (8 * i)).unwrap() | value;
    }

    value
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
      0x10 => self.bgxofs[0],
      0x12 => self.bgyofs[0],
      0x14 => self.bgxofs[1],
      0x16 => self.bgyofs[1],
      0x18 => self.bgxofs[2],
      0x1a => self.bgyofs[2],
      0x1c => self.bgxofs[3],
      0x1e => self.bgyofs[3],
      0x20 => self.bg_props[0].dx as u16,
      0x22 => self.bg_props[0].dmx as u16,
      0x24 => self.bg_props[0].dy as u16,
      0x26 => self.bg_props[0].dmy as u16,
      0x28 => self.bg_props[0].x as u16,
      0x2a => (self.bg_props[0].x >> 16) as u16,
      0x2c => self.bg_props[0].y as u16,
      0x2e => (self.bg_props[0].y >> 16) as u16,
      0x30 => self.bg_props[1].dx as u16,
      0x32 => self.bg_props[1].dmx as u16,
      0x34 => self.bg_props[1].dy as u16,
      0x36 => self.bg_props[1].dmy as u16,
      0x38 => self.bg_props[1].x as u16,
      0x3a => (self.bg_props[1].x >> 16) as u16,
      0x3c => self.bg_props[1].y as u16,
      0x3e => (self.bg_props[1].y >> 16) as u16,
      0x40 => self.winh[0].val,
      0x42 => self.winh[1].val,
      0x44 => self.winv[0].val,
      0x46 => self.winv[1].val,
      0x48 => self.winin.bits(),
      0x4a => self.winout.bits(),
      0x4c => 0, // TODO
      0x50 => self.bldcnt.value,
      0x52 => self.bldalpha.read(),
      0x54 => self.bldy.read(),
      0x56..=0x5f => 0,
      _ => panic!("invalid address given to engine read register method")
    }
  }

  pub fn on_end_vblank(&mut self) {
    for bg_prop in &mut self.bg_props {
      bg_prop.internal_x = bg_prop.x;
      bg_prop.internal_y = bg_prop.y;
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
      0x10 => self.bgxofs[0] = value & 0x1ff,
      0x12 => self.bgyofs[0] = value & 0x1ff,
      0x14 => self.bgxofs[1] = value & 0x1ff,
      0x16 => self.bgyofs[1] = value & 0x1ff,
      0x18 => self.bgxofs[2] = value & 0x1ff,
      0x1a => self.bgyofs[2] = value & 0x1ff,
      0x1c => self.bgxofs[3] = value & 0x1ff,
      0x1e => self.bgyofs[3] = value & 0x1ff,
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
      0x4c..=0x4e => (), // TODO
      0x50 => self.bldcnt.write(value),
      0x52 => self.bldalpha.write(value),
      0x54 => self.bldy.write(value),
      0x56..=0x5f => (),
      _ => panic!("invalid address given to engine write register method")
    }
  }
}