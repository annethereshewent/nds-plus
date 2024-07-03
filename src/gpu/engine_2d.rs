use std::path::Display;

use super::{registers::display_control_register::{DisplayControlRegister, DisplayMode}, vram::VRam, HEIGHT, WIDTH};

#[derive(Copy, Clone)]
pub struct Color {
  pub r: u8,
  pub g: u8,
  pub b: u8
}

impl Color {
  pub fn from(val: u16) -> Self {
    let r = (val & 0x1f) as u8;
    let g = ((val >> 5) & 0x1f) as u8;
    let b = ((val >> 10) & 0x1f) as u8;

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
  pub pixels: [u8; 3 * (WIDTH * HEIGHT) as usize]
}

impl<const IS_ENGINE_B: bool> Engine2d<IS_ENGINE_B> {
  pub fn new() -> Self {
    Self {
      dispcnt: DisplayControlRegister::new(),
      oam: [0; 0x400],
      pixels: [0; 3 * (WIDTH * HEIGHT) as usize]
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