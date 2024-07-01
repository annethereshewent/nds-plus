pub struct SPICNT {
  val: u32
}

impl SPICNT {
  pub fn new() -> Self {
    Self {
      val: 0
    }
  }

  pub fn write(&mut self, val: u32, mask: u32) {

  }
}