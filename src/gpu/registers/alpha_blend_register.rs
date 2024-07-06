pub struct AlphaBlendRegister {
  pub eva: u8,
  pub evb: u8
}

impl AlphaBlendRegister {
  pub fn new() -> Self {
    Self {
      eva: 0,
      evb: 0
    }
  }

  pub fn write(&mut self, value: u16) {
    self.eva = (value & 0b11111) as u8;
    self.evb = ((value >> 8) & 0b11111 ) as u8;
  }
}