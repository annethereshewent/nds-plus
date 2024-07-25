pub struct FogColorRegister {
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub alpha: u8
}

impl FogColorRegister {
  pub fn new() -> Self {
    Self {
      r: 0,
      g: 0,
      b: 0,
      alpha: 0
    }
  }

  pub fn write(&mut self, value: u32) {
    self.r = (value & 0x1f) as u8;
    self.g = ((value >> 5) & 0x1f) as u8;
    self.b = ((value >> 10) & 0x1f) as u8;

    self.alpha = ((value >> 16) & 0x1f) as u8;
  }
}