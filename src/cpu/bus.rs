use std::{cell::{Cell, RefCell}, rc::Rc};

use cp15::CP15;
use spi::SPI;

use super::{cycle_lookup_tables::CycleLookupTables, dma::dma_channels::{AddressType, DmaChannels}, registers::{interrupt_enable_register::InterruptEnableRegister, interrupt_request_register::InterruptRequestRegister, key_input_register::KeyInputRegister, waitstate_control_register::WaitstateControlRegister}, timers::Timers};

pub mod arm7;
pub mod arm9;
pub mod cp15;
pub mod spi;
pub mod flash;

pub const ITCM_SIZE: usize = 0x8000;
pub const DTCM_SIZE: usize = 0x4000;
const MAIN_MEMORY_SIZE: usize = 0x4_0000;
const WRAM_SIZE: usize = 0x1_0000;
const SHARED_WRAM_SIZE: usize = 0x8000;

pub struct Arm9Bus {
  timers: Timers<true>,
  dma_channels: Rc<RefCell<DmaChannels<true>>>,
  bios9: Vec<u8>,
  pub cp15: CP15,
  // TODO: add interrupt controllers
}

impl Arm9Bus {
  pub fn load_bios(&mut self, bytes: Vec<u8>) {
    self.bios9 = bytes;
  }
}
pub struct Arm7Bus {
  timers: Timers<false>,
  dma_channels: Rc<RefCell<DmaChannels<false>>>,
  pub bios7: Vec<u8>,
  pub wram: Box<[u8]>
  // TODO: interrupt controllers
}

impl Arm7Bus {
  pub fn load_bios(&mut self, bytes: Vec<u8>) {
    self.bios7 = bytes;
  }
}

pub struct Bus {
  pub arm9: Arm9Bus,
  pub arm7: Arm7Bus,
  pub is_halted: bool,
  itcm: Box<[u8]>,
  dtcm: Box<[u8]>,
  main_memory: Box<[u8]>,
  shared_wram: Box<[u8]>,
  pub spi: SPI
}

impl Bus {
  pub fn new(firmware_bytes: Vec<u8>, bios7_bytes: Vec<u8>, bios9_bytes: Vec<u8>) -> Self {
    let dma_channels7 = Rc::new(RefCell::new(DmaChannels::new()));
    let dma_channels9 = Rc::new(RefCell::new(DmaChannels::new()));
    let interrupt_request = Rc::new(Cell::new(InterruptRequestRegister::from_bits_retain(0)));

    Self {
      arm7: Arm7Bus {
        timers: Timers::new(interrupt_request.clone()),
        bios7: bios7_bytes,
        dma_channels: dma_channels7.clone(),
        wram: vec![0; WRAM_SIZE].into_boxed_slice()
      },
      arm9: Arm9Bus {
        timers: Timers::new(interrupt_request.clone()),
        bios9: bios9_bytes,
        dma_channels: dma_channels9.clone(),
        cp15: CP15::new()
      },
      is_halted: false,
      shared_wram: vec![0; SHARED_WRAM_SIZE].into_boxed_slice(),
      main_memory: vec![0; MAIN_MEMORY_SIZE].into_boxed_slice(),
      itcm: vec![0; ITCM_SIZE].into_boxed_slice(),
      dtcm: vec![0; DTCM_SIZE].into_boxed_slice(),
      spi: SPI::new(firmware_bytes)
    }
  }

  pub fn clear_interrupts(&mut self, value: u16) {

  }
}
