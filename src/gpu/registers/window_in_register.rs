bitflags! {
  pub struct WindowInRegister: u16 {
    const Window0ObjEnable = 0b1 << 4;
    const Window0ColorEffect = 0b1 << 5;
    const Window1ObjEnable = 0b1 << 12;
    const Window1ColorEffect = 0b1 << 13;
  }
}

impl WindowInRegister {
  pub fn window0_bg_enable(&self) -> u16 {
    self.bits() & 0b1111
  }

  pub fn window1_bg_enable(&self) -> u16 {
    (self.bits() >> 8) & 0b1111
  }
}