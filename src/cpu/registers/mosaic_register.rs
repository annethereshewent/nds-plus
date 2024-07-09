pub struct MosaicRegister {
  pub val: u16
}

impl MosaicRegister {
  pub fn new() -> Self {
    Self {
      val: 0
    }
  }
  pub fn write(&mut self, val: u16, mask: u16) {
    self.val = (self.val & mask) | val;
  }

  pub fn read(&self) -> u16 {
    self.val
  }
}