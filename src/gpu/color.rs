#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub alpha: Option<u8>
}

impl Color {
  pub fn new() -> Self {
    Self {
      r: 0,
      g: 0,
      b: 0,
      alpha: None
    }
  }

  pub fn convert(&mut self) -> Self {
    self.r = (self.r << 3) | (self.r >> 2);
    self.g = (self.g << 3) | (self.g >> 2);
    self.b = (self.b << 3) | (self.b >> 2);

    *self
  }

  pub fn alpha6(&self) -> u8 {
    let return_alpha = if let Some(alpha) = self.alpha {
      alpha
    } else {
      0x1f
    };

    Self::upscale(return_alpha)
  }

  pub fn write(&mut self, value: u16) {
    self.r = (value & 0x1f) as u8;
    self.g = ((value >> 5) & 0x1f) as u8;
    self.b = ((value >> 10) & 0x1f) as u8;
  }

  pub fn to_rgb5(&mut self) -> Self {
    self.r = self.r >> 1;
    self.g = self.g >> 1;
    self.b = self.b >> 1;

    *self
  }

  pub fn to_rgb6(&mut self) -> Self {
    self.r = Self::upscale(self.r);
    self.g = Self::upscale(self.g);
    self.b = Self::upscale(self.b);

    *self
  }

  pub fn blend_half(&self, color: Color) -> Color {
    let r = (self.r + color.r) / 2;
    let g = (self.g + color.g) / 2;
    let b = (self.b + color.b) / 2;

    Color {
      r,
      g,
      b,
      alpha: self.alpha
    }
  }

  pub fn blend_texture(&self, color: Color) -> Color {
    // // (color0 * 5 + color1 * 3) / 8
    let r = (self.r * 5 + color.r * 3) / 8;
    let g = (self.g * 5 + color.g * 3) / 8;
    let b = (self.b * 5 + color.b * 3) / 8;

    Color {
      r,
      g,
      b,
      alpha: self.alpha
    }
  }

  fn upscale(value: u8) -> u8 {
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
      b,
      alpha: None
    }
  }

  pub fn from(val: u16) -> Self {
    Color {
      r: (val & 0x1f) as u8,
      g: ((val >> 5) & 0x1f) as u8,
      b: ((val >> 10) & 0x1f) as u8,
      alpha: None
    }
  }
}