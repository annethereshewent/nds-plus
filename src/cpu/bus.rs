use std::{cell::{Cell, RefCell}, rc::Rc};

use super::{cycle_lookup_tables::CycleLookupTables, dma::dma_channels::{AddressType, DmaChannels}, registers::{interrupt_enable_register::InterruptEnableRegister, interrupt_request_register::InterruptRequestRegister, key_input_register::KeyInputRegister, waitstate_control_register::WaitstateControlRegister}, timers::Timers};


const ITCM_SIZE: usize = 0x8000;
const DTCM_SIZE: usize = 0x4000;
const MAIN_MEMORY_SIZE: usize = 0x4_0000;
const WRAM_SIZE: usize = 0x1_0000;
const SHARED_WRAM_SIZE: usize = 0x8000;

pub struct Arm9Bus {
  timers: Timers<true>,
  dma_channels: Rc<RefCell<DmaChannels<true>>>,
  bios9: Vec<u8>
  // TODO: add interrupt controllers
}

impl Arm9Bus {

}
pub struct Arm7Bus {
  timers: Timers<false>,
  dma_channels: Rc<RefCell<DmaChannels<false>>>,
  pub bios7: Vec<u8>,
  pub wram: Box<[u8]>
  // TODO: interrupt controllers
}

impl Arm7Bus {

}

pub struct Bus {
  pub arm9: Arm9Bus,
  pub arm7: Arm7Bus,
  pub is_halted: bool,
  itcm: Box<[u8]>,
  dtcm: Box<[u8]>,
  main_memory: Box<[u8]>,
  shared_wram: Box<[u8]>,
}

impl Bus {
  pub fn new() -> Self {
    let dma_channels7 = Rc::new(RefCell::new(DmaChannels::new()));
    let dma_channels9 = Rc::new(RefCell::new(DmaChannels::new()));
    let interrupt_request = Rc::new(Cell::new(InterruptRequestRegister::from_bits_retain(0)));

    Self {
      arm7: Arm7Bus {
        timers: Timers::new(interrupt_request.clone()),
        bios7: Vec::new(),
        dma_channels: dma_channels7.clone(),
        wram: vec![0; WRAM_SIZE].into_boxed_slice()
      },
      arm9: Arm9Bus {
        timers: Timers::new(interrupt_request.clone()),
        bios9: Vec::new(),
        dma_channels: dma_channels9.clone()
      },
      is_halted: false,
      shared_wram: vec![0; SHARED_WRAM_SIZE].into_boxed_slice(),
      main_memory: vec![0; MAIN_MEMORY_SIZE].into_boxed_slice(),
      itcm: vec![0; ITCM_SIZE].into_boxed_slice(),
      dtcm: vec![0; DTCM_SIZE].into_boxed_slice()
    }
  }
  pub fn mem_read_32(&mut self, address: u32) -> u32 {
    self.mem_read_16(address) as u32 | ((self.mem_read_16(address + 2) as u32) << 16)
  }

  pub fn mem_read_16(&mut self, address: u32) -> u16 {
    match address {
      0x400_0000..=0x4ff_ffff => self.io_read_16(address),
      _ => self.mem_read_8(address) as u16 | ((self.mem_read_8(address + 1) as u16) << 8)
    }
  }

  pub fn mem_read_8(&mut self, address: u32) -> u8 {
    match address {
      0x400_0000..=0x4ff_ffff => self.io_read_8(address),
      0x700_0000..=0x7ff_ffff => 0,
      0x800_0000..=0xdff_ffff => {
        0
      }
      _ => {
        panic!("reading from unsupported address: {:X}", address);
      }
    }
  }

  fn io_read_16(&mut self, address: u32) -> u16 {
    let address = if address & 0xfffe == 0x8000 {
      0x400_0800
    } else {
      address
    };

    match address {
      _ => {
        panic!("io register not implemented: {:X}", address);
      }
    }
  }

  fn io_read_8(&mut self, address: u32) -> u8 {
    let val = self.io_read_16(address & !(0b1));

    if address & 0b1 == 1 {
      (val >> 8) as u8
    } else {
      (val & 0xff) as u8
    }
  }

  pub fn mem_write_32(&mut self, address: u32, val: u32) {
    let upper = (val >> 16) as u16;
    let lower = (val & 0xffff) as u16;

    self.mem_write_16(address, lower);
    self.mem_write_16(address + 2, upper);
  }

  pub fn mem_write_16(&mut self, address: u32, val: u16) {
    let upper = (val >> 8) as u8;
    let lower = (val & 0xff) as u8;

    match address {
      0x400_0000..=0x4ff_ffff => self.io_write_16(address, val),
      _ => {
        self.mem_write_8(address, lower);
        self.mem_write_8(address + 1, upper);
      }
    }
  }

  pub fn mem_write_8(&mut self, address: u32, val: u8) {
    match address {
      0x400_0000..=0x4ff_ffff => self.io_write_8(address, val),
      0x500_0000..=0x5ff_ffff => self.mem_write_16(address & 0x3fe, (val as u16) * 0x101),
      _ => {
        panic!("writing to unsupported address: {:X}", address);
      }
    }
  }

  pub fn io_write_16(&mut self, address: u32, value: u16) {
    let address = if address & 0xfffe == 0x8000 {
      0x400_0800
    } else {
      address
    };

    match address {
      0x400_0006 => (),
      _ => {
        panic!("io register not implemented: {:X}", address)
      }
    }
  }

  pub fn clear_interrupts(&mut self, value: u16) {

  }

  pub fn io_write_8(&mut self, address: u32, value: u8) {
    let address = if address & 0xffff == 0x8000 {
      0x400_0800
    } else {
      address
    };

    // println!("im being called with address {:X}", address);

    match address {
      _ => {
        let mut temp = self.mem_read_16(address & !(0b1));

        temp = if address & 0b1 == 1 {
          (temp & 0xff) | (value as u16) << 8
        } else {
          (temp & 0xff00) | value as u16
        };

        self.mem_write_16(address & !(0b1), temp);
      }
    }

    // todo: implement sound
  }
}
