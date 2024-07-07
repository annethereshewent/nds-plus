use std::path::Display;

use super::{registers::{alpha_blend_register::AlphaBlendRegister, bg_control_register::BgControlRegister, brightness_register::BrightnessRegister, color_effects_register::ColorEffectsRegister, display_control_register::{DisplayControlRegister, DisplayMode}, master_brightness_register::MasterBrightnessRegister, window_horizontal_register::WindowHorizontalRegister, window_in_register::WindowInRegister, window_out_register::WindowOutRegister, window_vertical_register::WindowVerticalRegister}, vram::VRam, BgProps, HEIGHT, WIDTH};

#[derive(Copy, Clone)]
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
}

pub struct Engine2d<const IS_ENGINE_B: bool> {
  pub dispcnt: DisplayControlRegister<IS_ENGINE_B>,
  pub oam: [u8; 0x400],
  pub pixels: [u8; 3 * (WIDTH * HEIGHT) as usize],
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
  bg_lines: [[Option<(u8, u8, u8)>; WIDTH as usize]; 4],
  pub master_brightness: MasterBrightnessRegister,

}

impl<const IS_ENGINE_B: bool> Engine2d<IS_ENGINE_B> {
  pub fn new() -> Self {
    Self {
      dispcnt: DisplayControlRegister::new(),
      oam: [0; 0x400],
      pixels: [0; 3 * (WIDTH * HEIGHT) as usize],
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
      bg_lines: [[None; WIDTH as usize]; 4],
      master_brightness: MasterBrightnessRegister::new()
    }
  }

  pub fn render_line(&mut self, y: u16, vram: &mut VRam) {
    match self.dispcnt.display_mode {
      DisplayMode::Mode0 => println!("in mode 0"),
      DisplayMode::Mode1 => println!("in mode 1"),
      DisplayMode::Mode2 => {
        for x in 0..WIDTH {
          let index = 2 * (y as usize * WIDTH as usize + x as usize);
          let bank = vram.get_lcdc_bank(self.dispcnt.vram_block);

          let color = bank[index] as u16 | (bank[(index + 1) as usize] as u16) << 8;

          let color = Color::from(color);

          self.set_pixel(x as usize, y as usize, color);
        }
      },
      DisplayMode::Mode3 => println!("in mode 3"),
    }
  }

  pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
    let i: usize = 3 * (x + y * WIDTH as usize);

    self.pixels[i] = color.r;
    self.pixels[i + 1] = color.g;
    self.pixels[i + 2] = color.b;

  }

  pub fn read_register(&self, address: u32) -> u16 {
    println!("read address = {:x}", address);
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
      _ => panic!("invalid address given to engine read register method")
    }
  }

  pub fn write_register(&mut self, address: u32, value: u16) {
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

    println!("address = {:x}", address);

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
      0x4c => (), // TODO (but probably not lmao, mosaic is pointless)
      0x50 => self.bldcnt.write(value),
      0x52 => self.bldalpha.write(value),
      0x54 => self.bldy.write(value),
      _ => panic!("invalid address given to engine write register method")
    }
  }
}