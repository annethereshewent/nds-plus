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
}