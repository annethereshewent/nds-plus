#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DmaTiming {
  Immediately,
  Vblank,
  Hblank,
  Fifo,
  StartOfDisplay,
  DSCartridgeSlot,
  GBACartridgeSlot,
  GeometryCommandFifo,
  Wireless,
  MainMemoryDisplay
}

bitflags! {
  #[derive(Copy, Clone, Default)]
  pub struct DmaControlRegister: u32 {
    const DMA_REPEAT = 0b1 << 25;
    const DMA_TRANSFER_TYPE = 0b1 << 26;
    const IRQ_ENABLE = 0b1 << 30;
    const DMA_ENABLE = 0b1 << 31;
  }
}

impl DmaControlRegister {
  pub fn dest_addr_control(&self) -> u32 {
    (self.bits() >> 21) & 0x3
  }

  pub fn source_addr_control(&self) -> u32 {
    (self.bits() >> 23) & 0x3
  }

  pub fn word_count(&self) -> u32 {
    self.bits() & 0x1fffff
  }

  pub fn dma_start_timing(&self, is_arm9: bool) -> DmaTiming {
    if is_arm9 {
      match (self.bits() >> 27) & 0x7 {
        0 => DmaTiming::Immediately,
        1 => DmaTiming::Vblank,
        2 => DmaTiming::Hblank,
        3 => DmaTiming::StartOfDisplay,
        4 => DmaTiming::MainMemoryDisplay,
        5 => DmaTiming::DSCartridgeSlot,
        6 => DmaTiming::GBACartridgeSlot,
        7 => DmaTiming::GeometryCommandFifo,
        _ => unreachable!()
      }
    } else {
      match (self.bits() >> 28) & 0x3 {
        0 => DmaTiming::Immediately,
        1 => DmaTiming::Vblank,
        2 => DmaTiming::DSCartridgeSlot,
        3 => DmaTiming::Wireless, // or could be GBA cartridge depending on the channel
        _ => unreachable!()
      }
    }

  }
}