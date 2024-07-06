#[derive(Copy, Clone, PartialEq)]
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
  #[derive(Copy, Clone)]
  pub struct DmaControlRegister: u32 {
    const DMA_REPEAT = 0b1 << 9;
    const DMA_TRANSFER_TYPE = 0b1 << 10;
    const GAME_PAK_DRQ = 0b1 << 11;
    const IRQ_ENABLE = 0b1 << 14;
    const DMA_ENABLE = 0b1 << 15;
  }
}

impl DmaControlRegister {
  pub fn dest_addr_control(&self) -> u32 {
    (self.bits() >> 5) & 0b11
  }

  pub fn source_addr_control(&self) -> u32 {
    (self.bits() >> 7) & 0b11
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