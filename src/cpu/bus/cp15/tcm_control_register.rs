
pub struct TCMControlRegister {
  val: u32
}

impl TCMControlRegister {
  pub fn new(val: u32) -> Self {
    Self {
      val
    }
  }

  pub fn read(&self) -> u32 {
    self.val
  }

  pub fn base_address(&self) -> u32 {
    self.val & !0xfff
  }

  pub fn virtual_size_shift(&self) -> u32 {
    (self.val >> 1) & 0x1f
  }

  pub fn virtual_size(&self) -> u32 {
    0x200 << self.virtual_size_shift()
  }

  pub fn write(&mut self, val: u32) {
    self.val = val;
  }
}