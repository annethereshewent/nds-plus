use serde::{Deserialize, Serialize};

bitflags! {
  #[derive(Clone, Copy, Serialize, Deserialize)]
  #[serde(transparent)]
  pub struct ExternalKeyInputRegister: u16 {
    const BUTTON_X = 1;
    const BUTTON_Y = 1 << 1;
    const UNKNOWN1 = 1 << 2;
    const DEBUG_DOWN = 1 << 3;
    const UNKNOWN2 = 1 << 4;
    const UNKNOWN3 = 1 << 5;
    const PEN_DOWN = 1 << 6;
    const HINGE_OPEN = 1 << 7;
  }
}

impl ExternalKeyInputRegister {
  pub fn new() -> Self {
    Self::BUTTON_X | Self::BUTTON_Y | Self::PEN_DOWN | Self::DEBUG_DOWN | Self::UNKNOWN1 | Self::UNKNOWN2 | Self::UNKNOWN3
  }
}