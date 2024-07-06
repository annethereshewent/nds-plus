bitflags! {
  #[derive(Copy, Clone)]
  pub struct BgControlRegister: u16 {
    const MOSAIC = 0b1 << 6;
    const PALETTES = 0b1 << 7;
    const DISPLAY_AREA_OVERFLOW = 0b1 << 13;
  }
}

impl BgControlRegister {
  pub fn bg_priority(&self) -> u16 {
    self.bits() & 0b11
  }

  pub fn character_base_block(&self) -> u16 {
    (self.bits() >> 2) & 0b11
  }

  pub fn screen_base_block(&self) -> u16 {
    (self.bits() >> 8) & 0b11111
  }

  pub fn screen_size(&self) -> u16 {
    (self.bits() >> 14) & 0b11
  }

  pub fn get_screen_dimensions(&self) -> (u16, u16) {
    let size = (self.bits() >> 14) & 0b11;

    match size {
      0 => (256, 256),
      1 => (512, 256),
      2 => (256, 512),
      3 => (512, 512),
      _ => unreachable!("impossible")
    }
  }
}