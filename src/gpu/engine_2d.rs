use std::path::Display;

use super::{registers::display_control_register::{DisplayControlRegister, DisplayMode}, vram::VRam, HEIGHT, WIDTH};

#[derive(Copy, Clone)]
pub struct Color {
  r: u8,
  g: u8,
  b: u8
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
  pub pixels: [Color; (WIDTH * HEIGHT) as usize]
}

impl<const IS_ENGINE_B: bool> Engine2d<IS_ENGINE_B> {
  pub fn new() -> Self {
    Self {
      dispcnt: DisplayControlRegister::new(),
      oam: [0; 0x400],
      pixels: [Color::from(0); (WIDTH * HEIGHT) as usize]
    }
  }

  pub fn render_line(&mut self, y: u16, vram: &mut VRam) {
    match self.dispcnt.display_mode {
      DisplayMode::Mode0 => println!("in mode 0"),
      DisplayMode::Mode1 => println!("in mode 1"),
      DisplayMode::Mode2 => {

        for x in 0..WIDTH {
          let index = y * WIDTH + x;
          let bank = vram.get_lcdc_bank(self.dispcnt.vram_block);

          let color = bank[(index * 2) as usize] as u16 | (bank[((index * 2) + 1) as usize] as u16) << 8;

          self.pixels[(x + y * WIDTH) as usize] = Color::from(color);
        }
      },
      DisplayMode::Mode3 => println!("in mode 3"),
    }
  }
}