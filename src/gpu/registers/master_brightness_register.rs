pub struct MasterBrightnessRegister {
  val: u16
}

impl MasterBrightnessRegister {
  pub fn new() -> Self {
    Self {
      val: 0
    }
  }

  pub fn read(&self) -> u16 {
    self.val
  }

  pub fn write(&mut self, val: u16) {
    self.val = val;
  }
}