
pub struct CartridgeControlRegister {
  val: u32
}

impl CartridgeControlRegister {
  pub fn new() -> Self {
    Self {
      val: 0
    }
  }

  pub fn read(&self) -> u32 {
    self.val
  }

  pub fn write(&mut self, val: u32, mask: u32) {
    self.val &= mask;
    self.val |= val;

    // need to run commands here:
    println!("im awaiting commands");

  }
}