pub struct SpecularColor {
  r: u8,
  g: u8,
  b: u8,
  shininess_table_enable: bool
}

impl SpecularColor {
  pub fn new() -> Self {
    Self {
      r: 0,
      g: 0,
      b: 0,
      shininess_table_enable: false
    }
  }

  pub fn write(&mut self, value: u16) {
    self.r = (value & 0x1f) as u8;
    self.g = ((value >> 5) & 0x1f) as u8;
    self.b = ((value >> 10) & 0x1f) as u8;
    self.shininess_table_enable = (value >> 15) & 0b1 == 1;
  }
}