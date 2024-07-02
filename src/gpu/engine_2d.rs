use super::registers::display_control_register::DisplayControlRegister;

pub struct Engine2d<const IS_ENGINE_B: bool> {
  pub dispcnt: DisplayControlRegister<IS_ENGINE_B>,
  pub oam: [u8; 0x400]
}

impl<const IS_ENGINE_B: bool> Engine2d<IS_ENGINE_B> {
  pub fn new() -> Self {
    Self {
      dispcnt: DisplayControlRegister::new(),
      oam: [0; 0x400]
    }
  }
}