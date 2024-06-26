
bitflags! {
  #[derive(Copy, Clone)]
  pub struct DmaControlRegister: u16 {
    const DMA_REPEAT = 0b1 << 9;
    const DMA_TRANSFER_TYPE = 0b1 << 10;
    const GAME_PAK_DRQ = 0b1 << 11;
    const IRQ_ENABLE = 0b1 << 14;
    const DMA_ENABLE = 0b1 << 15;
  }
}

impl DmaControlRegister {
  pub fn dest_addr_control(&self) -> u16 {
    (self.bits() >> 5) & 0b11
  }

  pub fn source_addr_control(&self) -> u16 {
    (self.bits() >> 7) & 0b11
  }

  pub fn dma_start_timing(&self) -> u16 {
    (self.bits() >> 12) & 0b11
  }
}