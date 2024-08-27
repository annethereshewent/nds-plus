use crate::gpu::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub struct Touchscreen {
  pub x: u16,
  pub y: u16,

  data: u16,
  return_byte: u8
}

impl Touchscreen {
  pub fn new() -> Self {
    Self {
      x: 0,
      y: 0,

      data: 0,
      return_byte: 0
    }
  }

  pub fn write(&mut self, value: u8) {
    self.return_byte = (self.data >> 8) as u8;

    self.data <<= 8;


    if (value >> 7) & 0b1 == 1 {
      // start bit must be set to write the data
      let channel = (value >> 4) & 0x7;
      self.data = match channel {
        1 => self.y << 3,
        5 => self.x << 3,
        6 => 0,
        _ => 0xfff
      }
    }
  }

  pub fn deselect(&mut self) {
    self.data = 0;
  }

  pub fn read(&self) -> u8 {
    self.return_byte
  }

  pub fn touch_screen(&mut self, x: u16, y: u16) {
    self.x = x << 4;
    self.y = y << 4;
  }

  pub fn touch_screen_controller(&mut self, x: i16, y: i16) {
    let middle_x = SCREEN_WIDTH  as i16/ 2;
    let middle_y = SCREEN_HEIGHT as i16 / 2;

    let pointer_x = x >> 10;
    let pointer_y = y >> 10;

    self.x = ((middle_x + pointer_x) << 4) as u16;
    self.y = ((middle_y + pointer_y) << 4) as u16;
  }


  pub fn release_screen(&mut self) {
    self.x = 0;
    self.y = 0xfff;
  }
}