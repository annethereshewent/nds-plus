bitflags! {
  pub struct WindowOutRegister: u16 {
    const OutsideWindowObjEnable = 0b1 << 4;
    const OutsideWindowColorEffect = 0b1 << 5;
    const ObjWindowObjEnable = 0b1 << 12;
    const ObjWIndowColorEffect = 0b1 << 13;
  }
}

impl WindowOutRegister {
  pub fn outside_window_background_enable_bits(&self) -> u16 {
    self.bits() & 0b1111
  }

  pub fn obj_window_bg_enable_bits(&self) -> u16 {
    (self.bits() >> 8) & 0b1111
  }
}