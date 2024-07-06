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
        _ => 0x7ff8
      }
    }
  }

  pub fn read(&self) -> u8 {
    self.return_byte
  }
}