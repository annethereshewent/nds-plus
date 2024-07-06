pub struct ColorEffectsRegister {
  pub bg_first_pixels: [bool; 4],
  pub bg_second_pixels: [bool; 4],
  pub obj_first_pixel: bool,
  pub obj_second_pixel: bool,
  pub backdrop_first_pixel: bool,
  pub backdrop_second_pixel: bool,
  pub color_effect: ColorEffect,
  pub value: u16
}

impl ColorEffectsRegister {
  pub fn new() -> Self {
    Self {
      bg_first_pixels: [false; 4],
      bg_second_pixels: [false; 4],
      obj_first_pixel: false,
      obj_second_pixel: false,
      backdrop_second_pixel: false,
      backdrop_first_pixel: false,
      color_effect: ColorEffect::None,
      value: 0
    }
  }
  pub fn write(&mut self, value: u16) {
    self.value = value;
    let color_effect = (value >> 6) & 0b11;

    self.color_effect = match color_effect {
      0 => ColorEffect::None,
      1 => ColorEffect::AlphaBlending,
      2 => ColorEffect::Brighten,
      3 => ColorEffect::Darken,
      _ => unreachable!("can't happen")
    };

    for i in 0..4 {
      self.bg_first_pixels[i] = (value >> i) & 0b1 == 1;
    }

    self.obj_first_pixel = (value >> 4) & 0b1 == 1;
    self.backdrop_first_pixel = (value >> 5) & 0b1 == 1;

    for i in 0..4 {
      self.bg_second_pixels[i] = (value >> (i + 8)) & 0b1 == 1;
    }

    self.obj_second_pixel = (value >> 12) & 0b1 == 1;
    self.backdrop_second_pixel = (value >> 13) & 0b1 == 1;
  }
}

pub enum ColorEffect {
  None = 0,
  AlphaBlending = 1,
  Brighten = 2,
  Darken = 3
}