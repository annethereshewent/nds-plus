use std::{cell::{Cell, RefCell}, rc::Rc};

use cartridge::{Cartridge, CHIP_ID};
use cp15::CP15;
use spi::SPI;
use wram_control_register::WRAMControlRegister;

use super::{cycle_lookup_tables::CycleLookupTables, dma::dma_channels::{AddressType, DmaChannels}, registers::{interrupt_enable_register::InterruptEnableRegister, interrupt_request_register::InterruptRequestRegister, key_input_register::KeyInputRegister, waitstate_control_register::WaitstateControlRegister}, timers::Timers};

pub mod arm7;
pub mod arm9;
pub mod cp15;
pub mod spi;
pub mod flash;
pub mod cartridge;
pub mod wram_control_register;
pub mod wram_status_register;

pub const ITCM_SIZE: usize = 0x8000;
pub const DTCM_SIZE: usize = 0x4000;
const MAIN_MEMORY_SIZE: usize = 0x40_0000;
const WRAM_SIZE: usize = 0x1_0000;
const SHARED_WRAM_SIZE: usize = 0x8000;

pub struct Arm9Bus {
  timers: Timers<true>,
  dma_channels: Rc<RefCell<DmaChannels<true>>>,
  bios9: Vec<u8>,
  pub cp15: CP15,
  pub postflg: bool,
  pub interrupt_master_enable: bool
  // TODO: add interrupt controllers
}

pub struct Arm7Bus {
  timers: Timers<false>,
  dma_channels: Rc<RefCell<DmaChannels<false>>>,
  pub bios7: Vec<u8>,
  pub wram: Box<[u8]>,
  pub postflg: bool,
  pub interrupt_master_enable: bool
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
  pub spi: SPI,
  pub cartridge: Cartridge,
  pub wramcnt: WRAMControlRegister
}

impl Bus {
  pub fn new(firmware_bytes: Vec<u8>, bios7_bytes: Vec<u8>, bios9_bytes: Vec<u8>, rom_bytes: Vec<u8>, skip_bios: bool) -> Self {
    let dma_channels7 = Rc::new(RefCell::new(DmaChannels::new()));
    let dma_channels9 = Rc::new(RefCell::new(DmaChannels::new()));
    let interrupt_request = Rc::new(Cell::new(InterruptRequestRegister::from_bits_retain(0)));

    let mut bus = Self {
      arm7: Arm7Bus {
        timers: Timers::new(interrupt_request.clone()),
        bios7: bios7_bytes,
        dma_channels: dma_channels7.clone(),
        wram: vec![0; WRAM_SIZE].into_boxed_slice(),
        postflg: skip_bios,
        interrupt_master_enable: false
      },
      arm9: Arm9Bus {
        timers: Timers::new(interrupt_request.clone()),
        bios9: bios9_bytes,
        dma_channels: dma_channels9.clone(),
        cp15: CP15::new(),
        postflg: skip_bios,
        interrupt_master_enable: false
      },
      is_halted: false,
      shared_wram: vec![0; SHARED_WRAM_SIZE].into_boxed_slice(),
      main_memory: vec![0; MAIN_MEMORY_SIZE].into_boxed_slice(),
      itcm: vec![0; ITCM_SIZE].into_boxed_slice(),
      dtcm: vec![0; DTCM_SIZE].into_boxed_slice(),
      spi: SPI::new(firmware_bytes),
      cartridge: Cartridge::new(rom_bytes),
      wramcnt: WRAMControlRegister::new()
    };

    if skip_bios {
      bus.skip_bios();
    }

    bus
  }

  pub fn clear_interrupts(&mut self, value: u16) {

  }

  fn skip_bios(&mut self) {
    // load header into RAM starting at address 0x27ffe00 (per the docs)
    let address = 0x27ffe00 & (MAIN_MEMORY_SIZE - 1);

    self.main_memory[address..address + 0x170].copy_from_slice(&self.cartridge.rom[0..0x170]);

    let arm9_rom_address = self.cartridge.header.arm9_rom_offset;
    let arm9_ram_address = self.cartridge.header.arm9_ram_address;
    let arm9_size = self.cartridge.header.arm9_size;

    // load rom into memory
    self.load_rom_into_memory(arm9_rom_address, arm9_ram_address, arm9_size, true);

    let arm7_rom_address = self.cartridge.header.arm7_rom_offset;
    let arm7_ram_address = self.cartridge.header.arm7_ram_address;
    let arm7_size = self.cartridge.header.arm7_size;

    self.load_rom_into_memory(arm7_rom_address, arm7_ram_address, arm7_size, false);

    // set hardcoded values (required for games to boot)
    self.write_mirrored_values(0x27ff800);
    self.write_mirrored_values(0x27ffc00);

    // write the rest of the hardcoded values
    self.arm9_mem_write_16(0x027ff850, 0x5835);
    self.arm9_mem_write_16(0x027ffc10, 0x5835);
    self.arm9_mem_write_16(0x027ffc30, 0xffff);
    self.arm9_mem_write_16(0x027ffc40, 0x1);

    self.arm9_mem_write_8(0x23FFC80, 0x5);

  }

  fn write_mirrored_values(&mut self, base_address: u32) {
    self.arm9_mem_write_32(base_address, CHIP_ID);
    self.arm9_mem_write_32(base_address + 0x4, CHIP_ID);
    self.arm9_mem_write_16(base_address + 0x8, self.cartridge.rom[0x15e] as u16 | (self.cartridge.rom[0x15f] as u16) << 8);
    self.arm9_mem_write_16(base_address + 0xa, self.cartridge.rom[0x6c] as u16 | (self.cartridge.rom[0x6d] as u16) << 8);
  }

  fn load_rom_into_memory(&mut self, rom_address: u32, ram_address: u32, size: u32, is_arm9: bool) {
    for i in 0..size {
      if is_arm9 {
        self.arm9_mem_write_8(ram_address+i, self.cartridge.rom[(rom_address + i) as usize]);
      } else {
        self.arm7_mem_write_8(ram_address+i, self.cartridge.rom[(rom_address + i) as usize])
      }
    }
  }
}
