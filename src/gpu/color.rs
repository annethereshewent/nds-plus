#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
  pub r: u8,
  pub g: u8,
  pub b: u8
}

impl Color {
  pub fn new() -> Self {
    Self {
      r: 0,
      g: 0,
      b: 0
    }
  }

  pub fn convert(&mut self) -> Self {
    self.r = (self.r << 3) | (self.r >> 2);
    self.g = (self.g << 3) | (self.g >> 2);
    self.b = (self.b << 3) | (self.b >> 2);

    *self
  }

  pub fn write(&mut self, value: u16) {
    self.r = (value & 0x1f) as u8;
    self.g = ((value >> 5) & 0x1f) as u8;
    self.b = ((value >> 10) & 0x1f) as u8;
  }

  pub fn to_rgb6(&mut self) {
    self.r = Self::to_rgb6_internal(self.r);
    self.g = Self::to_rgb6_internal(self.g);
    self.b = Self::to_rgb6_internal(self.b);
  }

  fn to_rgb6_internal(value: u8) -> u8 {
    if value == 0 {
      return 0;
    }

    value * 2 + 1
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