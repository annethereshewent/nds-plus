bitflags! {
  #[derive(Copy, Clone)]
  pub struct InterruptRequestRegister: u16 {
    const VBLANK = 0b1;
    const HBLANK = 0b1 << 1;
    const VCOUNTER_MATCH = 0b1 << 2;
    const TIMER_0_OVERFLOW = 0b1 << 3;
    const TIMER_1_OVERFLOW = 0b1 << 4;
    const TIMER_2_OVERFLOW = 0b1 << 5;
    const TIMER_3_OVERFLOW = 0b1 << 6;
    const SERIAL_COMM = 0b1 << 7;
    const DMA0 = 0b1 << 8;
    const DMA1 = 0b1 << 9;
    const DMA2 = 0b1 << 10;
    const DMA3 = 0b1 << 11;
    const KEYPAD = 0b1 << 12;
    const GAMEPACK = 0b1 << 13;
  }
}

impl InterruptRequestRegister {
  pub fn request_dma(&mut self, id: usize) {
    match id {
      0 => self.insert(Self::DMA0),
      1 => self.insert(Self::DMA1),
      2 => self.insert(Self::DMA2),
      3 => self.insert(Self::DMA3),
      _ => panic!("invalid id specified for dma interrupt: {id}")
    }
  }

  pub fn request_timer(&mut self, id: usize) {
    match id {
      0 => self.insert(Self::TIMER_0_OVERFLOW),
      1 => self.insert(Self::TIMER_1_OVERFLOW),
      2 => self.insert(Self::TIMER_2_OVERFLOW),
      3 => self.insert(Self::TIMER_3_OVERFLOW),
      _ => panic!("invalid id specified for dma interrupt: {id}")
    }
  }
}