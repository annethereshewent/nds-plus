bitflags! {
  #[derive(Clone, Copy)]
  pub struct ExternalKeyInputRegister: u16 {
    const BUTTON_X = 1;
    const BUTTON_Y = 1 << 1;
    const DEBUG_DOWN = 1 << 3;
    const PEN_DOWN = 1 << 6;
    const HINGE_OPEN = 1 << 7;
  }
}

impl ExternalKeyInputRegister {
  pub fn new() -> Self {
    Self::BUTTON_X | Self::BUTTON_Y | Self::PEN_DOWN | Self::DEBUG_DOWN
  }
}