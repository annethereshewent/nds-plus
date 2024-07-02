#[derive(Copy, Clone, Debug)]
pub struct VramControlRegister {
  val: u8,
  id: usize,
  pub vram_mst: u8,
  pub vram_offset: u8,
  pub vram_enable: bool
}

impl VramControlRegister {
  pub fn new(id: usize) -> Self {
    Self {
      id,
      val: 0,
      vram_enable: false,
      vram_offset: 0,
      vram_mst: 0
    }
  }

  pub fn write(&mut self, val: u8) {
    let abhi_registers: [usize; 4] = [0, 1, 7, 8];
    let ehi_registers: [usize; 3] = [4, 7, 8];

    // bit 2 not used by A,B,H,I
    self.vram_mst = if !abhi_registers.contains(&self.id) {
      val & 0x7
    } else {
      val & 0x3
    };

    if !ehi_registers.contains(&self.id) {
      self.vram_offset = (val >> 3) & 0x3;
    }

    self.vram_enable = (val >> 7) & 0b1 == 1;
  }
}