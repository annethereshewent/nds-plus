bitflags! {
  #[derive(Copy, Clone)]
  pub struct DispStatFlags: u16 {
    const VBLANK = 1;
    const HBLANK = 1 << 1;
    const VCOUNTER = 1 << 2;
    const VBLANK_IRQ_ENABLE = 1 << 3;
    const HBLANK_IRQ_ENABLE = 1 << 4;
    const VCOUNTER_IRQ_ENABLE = 1 << 5;
  }
}

pub struct DisplayStatusRegister {
  pub flags: DispStatFlags,
  pub vcount_setting: u16
}

impl DisplayStatusRegister {
  pub fn new() -> Self {
    Self {
      flags: DispStatFlags::from_bits_retain(0),
      vcount_setting: 0,
    }
  }

  pub fn write(&mut self, val: u16) {
    self.flags = DispStatFlags::from_bits_retain(val);

    self.vcount_setting = val >> 8;

    self.vcount_setting |= ((val >> 7) & 0b1) << 8;
  }

  pub fn read(&self) -> u16 {
    let vcount_msb = (self.vcount_setting >> 8) & 0b1;
    let vcount_lsbs = (self.vcount_setting) & 0xff;

    let val = self.flags.bits() | (vcount_msb << 7) | (vcount_lsbs << 8);

    val
  }
}