use std::{cell::{Cell, Ref, RefCell}, rc::Rc};

use cartridge::{Cartridge, CHIP_ID};
use cp15::CP15;
use num_integer::Roots;
use spi::SPI;
use wram_control_register::WRAMControlRegister;

use crate::{gpu::GPU, scheduler::Scheduler};

use super::{dma::{dma_channel::{registers::dma_control_register::DmaControlRegister, DmaParams}, dma_channels::DmaChannels}, registers::{division_control_register::{DivisionControlRegister, DivisionMode}, external_memory::ExternalMemory, interrupt_enable_register::InterruptEnableRegister, interrupt_request_register::InterruptRequestRegister, ipc_fifo_control_register::{IPCFifoControlRegister, FIFO_CAPACITY}, ipc_sync_register::IPCSyncRegister, key_input_register::KeyInputRegister, square_root_control_register::{BitMode, SquareRootControlRegister}}, timers::Timers, MemoryAccess};

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
  pub timers: Timers,
  pub dma_channels: DmaChannels,
  bios9: Vec<u8>,
  pub cp15: CP15,
  pub postflg: bool,
  pub interrupt_master_enable: bool,
  pub ipcsync: IPCSyncRegister,
  pub ipcfifocnt: IPCFifoControlRegister,
  pub interrupt_request: InterruptRequestRegister,
  pub interrupt_enable: InterruptEnableRegister,
  sqrtcnt: SquareRootControlRegister,
  divcnt: DivisionControlRegister,
  div_numerator: u64,
  div_denomenator: u64,
  div_result: u64,
  div_remainder: u64,
  sqrt_param: u64,
  sqrt_result: u32
}

pub struct Arm7Bus {
  pub timers: Timers,
  pub dma_channels: DmaChannels,
  pub bios7: Vec<u8>,
  pub wram: Box<[u8]>,
  pub postflg: bool,
  pub interrupt_master_enable: bool,
  pub ipcsync: IPCSyncRegister,
  pub ipcfifocnt: IPCFifoControlRegister,
  pub interrupt_request: InterruptRequestRegister,
  pub interrupt_enable: InterruptEnableRegister
}

pub struct Bus {
  pub arm9: Arm9Bus,
  pub arm7: Arm7Bus,
  pub is_halted: bool,
  pub gpu: GPU,
  itcm: Box<[u8]>,
  dtcm: Box<[u8]>,
  main_memory: Box<[u8]>,
  shared_wram: Box<[u8]>,
  pub spi: SPI,
  pub cartridge: Cartridge,
  pub wramcnt: WRAMControlRegister,
  pub key_input_register: KeyInputRegister,
  pub scheduler: Scheduler,
  exmem: ExternalMemory
}

impl Bus {
  pub fn new(
     firmware_bytes: Vec<u8>,
     bios7_bytes: Vec<u8>,
     bios9_bytes: Vec<u8>,
     rom_bytes: Vec<u8>,
     skip_bios: bool) -> Self
  {
    let dma_channels7 = DmaChannels::new(false);
    let dma_channels9 = DmaChannels::new(true);

    let mut scheduler = Scheduler::new();

    let mut bus = Self {
      arm7: Arm7Bus {
        timers: Timers::new(false),
        bios7: bios7_bytes,
        dma_channels: dma_channels7,
        wram: vec![0; WRAM_SIZE].into_boxed_slice(),
        postflg: skip_bios,
        interrupt_master_enable: false,
        ipcsync: IPCSyncRegister::new(),
        ipcfifocnt: IPCFifoControlRegister::new(),
        interrupt_request: InterruptRequestRegister::from_bits_retain(0),
        interrupt_enable: InterruptEnableRegister::from_bits_retain(0)
      },
      arm9: Arm9Bus {
        timers: Timers::new(true),
        bios9: bios9_bytes,
        dma_channels: dma_channels9,
        cp15: CP15::new(),
        postflg: skip_bios,
        interrupt_master_enable: false,
        ipcsync: IPCSyncRegister::new(),
        ipcfifocnt: IPCFifoControlRegister::new(),
        interrupt_request: InterruptRequestRegister::from_bits_retain(0),
        interrupt_enable: InterruptEnableRegister::from_bits_retain(0),
        sqrtcnt: SquareRootControlRegister::new(),
        sqrt_param: 0,
        sqrt_result: 0,
        divcnt: DivisionControlRegister::new(),
        div_denomenator: 0,
        div_numerator: 0,
        div_result: 0,
        div_remainder: 0
      },
      is_halted: false,
      shared_wram: vec![0; SHARED_WRAM_SIZE].into_boxed_slice(),
      main_memory: vec![0; MAIN_MEMORY_SIZE].into_boxed_slice(),
      itcm: vec![0; ITCM_SIZE].into_boxed_slice(),
      dtcm: vec![0; DTCM_SIZE].into_boxed_slice(),
      spi: SPI::new(firmware_bytes),
      cartridge: Cartridge::new(rom_bytes),
      wramcnt: WRAMControlRegister::new(),
      gpu: GPU::new(&mut scheduler),
      scheduler: scheduler,
      key_input_register: KeyInputRegister::from_bits_retain(0xffff),
      exmem: ExternalMemory::new()
    };

    if skip_bios {
      bus.skip_bios();
    }

    bus
  }

  fn handle_dma(&mut self, params: &mut DmaParams) -> u32 {
    let mut access = MemoryAccess::NonSequential;
    let mut cpu_cycles = 0;
    if params.fifo_mode {
      for _ in 0..4 {
        let (value, cycles) = self.load_32(params.source_address & !(0b11), access, true);

        cpu_cycles += cycles;

        cpu_cycles += self.store_32(params.destination_address & !(0b11), value, access, true);

        access = MemoryAccess::Sequential;
        params.source_address += 4;
      }
    } else if params.word_size == 4 {
      for _ in 0..params.count {
        let (word, cycles) = self.load_32(params.source_address & !(0b11), access, true);

        cpu_cycles += cycles;

        cpu_cycles += self.store_32(params.destination_address & !(0b11), word, access, true);

        access = MemoryAccess::Sequential;
        params.source_address = (params.source_address as i32).wrapping_add(params.source_adjust) as u32;
        params.destination_address = (params.destination_address as i32).wrapping_add(params.destination_adjust) as u32;
      }
    } else {
      for _ in 0..params.count {
        let (half_word, cycles) = self.load_16(params.source_address & !(0b1), access, true);

        cpu_cycles += cycles;

        cpu_cycles += self.store_16(params.destination_address & !(0b1), half_word, access, true);
        access = MemoryAccess::Sequential;
        params.source_address = (params.source_address as i32).wrapping_add(params.source_adjust) as u32;
        params.destination_address = (params.destination_address as i32).wrapping_add(params.destination_adjust) as u32;
      }
    }


    cpu_cycles
  }

  pub fn check_dma(&mut self, is_arm9: bool) -> u32 {
    let mut cpu_cycles = 0;

    // Rust *REALLY* makes it hard to keep your code DRY. like.... REALLY. This is really shitty annoying code
    // but if I don't do it like this rust will throw a fit. TODO: clean this code up somehow
    if is_arm9 && self.arm9.dma_channels.has_pending_transfers() {
      let mut dma_params = self.arm9.dma_channels.get_transfer_parameters();

      for i in 0..4 {
        if let Some(params) = &mut dma_params[i] {
          cpu_cycles += self.handle_dma(params);

          // update internal destination and source address for the dma channel as well.
          let channel = &mut self.arm9.dma_channels.channels[i];

          channel.internal_destination_address = params.destination_address;
          channel.internal_source_address = params.source_address;

          if channel.dma_control.contains(DmaControlRegister::DMA_REPEAT) {
            if channel.dma_control.dest_addr_control() == 3 {
              channel.internal_destination_address = channel.destination_address;
            }
          } else {
            channel.running = false;
            channel.dma_control.remove(DmaControlRegister::DMA_ENABLE);
          }

          if params.should_trigger_irq {
            self.arm9.interrupt_request.request_dma(i);
          }
        }
      }
    } else if !is_arm9 && self.arm7.dma_channels.has_pending_transfers() {
      let mut dma_params = self.arm7.dma_channels.get_transfer_parameters();

      for i in 0..4 {
        if let Some(params) = &mut dma_params[i] {
          cpu_cycles += self.handle_dma(params);

          // update internal destination and source address for the dma channel as well.
          let channel = &mut self.arm9.dma_channels.channels[i];

          channel.internal_destination_address = params.destination_address;
          channel.internal_source_address = params.source_address;

          if channel.dma_control.contains(DmaControlRegister::DMA_REPEAT) {
            if channel.dma_control.dest_addr_control() == 3 {
              channel.internal_destination_address = channel.destination_address;
            }
          } else {
            channel.running = false;
            channel.dma_control.remove(DmaControlRegister::DMA_ENABLE);
          }

          if params.should_trigger_irq {
            self.arm7.interrupt_request.request_dma(i);
          }
        }
      }
    }

    cpu_cycles
  }

  // these are similar to the cpu methods but only to be used with dma
  pub fn load_32(&mut self, address: u32, access: MemoryAccess, is_arm9: bool) -> (u32, u32) {
    // TODO: write this method
    // self.get_cycles(address, access, MemoryWidth::Width32);

    let cpu_cycles = 1;

    if !is_arm9 {
      (self.arm7_mem_read_32(address), cpu_cycles)
    } else {
      (self.arm9_mem_read_32(address), cpu_cycles)
    }
  }

  pub fn load_16(&mut self, address: u32, access: MemoryAccess, is_arm9: bool) -> (u16, u32) {
    // TODO: write this method
    // self.get_cycles(address, access, MemoryWidth::Width16);

    let cpu_cycles = 1;

    if !is_arm9 {
      (self.arm7_mem_read_16(address), cpu_cycles)
    } else {
      (self.arm9_mem_read_16(address), cpu_cycles)
    }
  }

  pub fn load_8(&mut self, address: u32, access: MemoryAccess, is_arm9: bool) -> (u8, u32) {
    // TODO: write this method
    // self.get_cycles(address, access, MemoryWidth::Width8);

    let cpu_cycles = 1;

    if !is_arm9 {
      (self.arm7_mem_read_8(address), cpu_cycles)
    } else {
      (self.arm9_mem_read_8(address), cpu_cycles)
    }
  }

  pub fn store_8(&mut self, address: u32, value: u8, access: MemoryAccess, is_arm9: bool) -> u32 {
    // TODO
    // self.get_cycles(address, access, MemoryWidth::Width8);

    let cpu_cycles = 1;

    if !is_arm9 {
      self.arm7_mem_write_8(address, value);
    } else {
      self.arm9_mem_write_8(address, value);
    }

    cpu_cycles
  }

  pub fn store_16(&mut self, address: u32, value: u16, access: MemoryAccess, is_arm9: bool) -> u32 {
    // TODO
    // self.get_cycles(address, access, MemoryWidth::Width8)

    let cpu_cycles = 1;

    if !is_arm9 {
      self.arm7_mem_write_16(address, value);
    } else {
      self.arm9_mem_write_16(address, value);
    }

    cpu_cycles
  }

  pub fn store_32(&mut self, address: u32, value: u32, access: MemoryAccess, is_arm9: bool) -> u32 {
    // TODO
    // self.get_cycles(address, access, MemoryWidth::Width8);

    let cpu_cycles = 1;

    if !is_arm9 {
      self.arm7_mem_write_32(address, value);
    } else {
      self.arm9_mem_write_32(address, value);
    }

    cpu_cycles
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

  pub fn write_sqrtcnt(&mut self, value: u16) {
    self.arm9.sqrtcnt.write(value);
  }

  pub fn start_sqrt_calculation(&mut self) -> u32 {
    let value = if self.arm9.sqrtcnt.mode() == BitMode::Bit32 {
      (self.arm9.sqrt_param as u32).sqrt()
    } else {
      self.arm9.sqrt_param.sqrt() as u32
    };

    value
  }

  pub fn start_div_calculation(&mut self) {
    if self.arm9.div_denomenator == 0 {
      self.arm9.divcnt.set_division_by_zero(true);
    } else {
      self.arm9.divcnt.set_division_by_zero(false);
    }

    let mut result: i64 = 0;
    let mut remainder: i64 = 0;

    let (numerator, denomenator) = match self.arm9.divcnt.mode() {
      DivisionMode::Mode0 => {
       ((self.arm9.div_numerator as u32 as i32 as i64), ((self.arm9.div_denomenator as u32 as i32 as i64)))
      }
      DivisionMode::Mode1 => {
        ((self.arm9.div_numerator as i64), (self.arm9.div_denomenator as u32 as i32 as i64))
      }
      DivisionMode::Mode2 => {
        (self.arm9.div_numerator as i64, self.arm9.div_denomenator as i64)
      }
    };

    if denomenator == 0 {
      remainder = numerator;
      if numerator == 0 {
        result = -1
      } else {
        result = if numerator < 0 {
          1
        } else {
          -1
        }
      }

      self.arm9.div_result = result as u64;
      self.arm9.div_remainder = remainder as u64;

      // overflows occur on div0 as well
      if self.arm9.divcnt.mode() == DivisionMode::Mode0 {
        // on 32 bit values invert the upper 32 bit values of the result
        self.arm9.div_result ^= 0xffffffff00000000
      }
    } else if numerator == i64::MIN && denomenator == -1 {
      // overflows
      self.arm9.div_result = numerator as u64;
      self.arm9.div_remainder = 0;

      if self.arm9.divcnt.mode() == DivisionMode::Mode0 {
        self.arm9.div_result ^= 0xffffffff00000000
      }

    } else {
      self.arm9.div_result = (numerator / denomenator) as u64;
      self.arm9.div_remainder = (numerator % denomenator) as u64;
    }
  }

  pub fn send_to_fifo(&mut self, is_arm9: bool, val: u32) {
    let (receive_control, send_control, interrupt_request) = if is_arm9 {
      (&mut self.arm7.ipcfifocnt, &mut self.arm9.ipcfifocnt, &mut self.arm7.interrupt_request)
    } else {
      (&mut self.arm9.ipcfifocnt, &mut self.arm7.ipcfifocnt, &mut self.arm9.interrupt_request)
    };

    if send_control.enabled {
      if receive_control.enabled && receive_control.receive_not_empty_irq && !send_control.fifo.is_empty() {
        interrupt_request.insert(InterruptRequestRegister::IPC_RECV_FIFO_NOT_EMPTY)
      }

      if send_control.fifo.len() == FIFO_CAPACITY {
        send_control.error = true;
      } else {
        send_control.fifo.push_back(val);
      }
    }
  }

  pub fn receive_from_fifo(&mut self, is_arm9: bool) -> u32 {
    let (receive_control, send_control, interrupt_request) = if is_arm9 {
      (&mut self.arm9.ipcfifocnt, &mut self.arm7.ipcfifocnt, &mut self.arm7.interrupt_request)
    } else {
      (&mut self.arm7.ipcfifocnt, &mut self.arm9.ipcfifocnt, &mut self.arm9.interrupt_request)
    };

    let previous_value = &mut send_control.previous_value;

    if receive_control.enabled {
      if let Some(value) = send_control.fifo.pop_front() {

        *previous_value = value;
        if send_control.enabled && send_control.send_empty_irq && send_control.fifo.is_empty() {
          interrupt_request.insert(InterruptRequestRegister::IPC_SEND_FIFO_EMPTY);
        }
        value
      } else {
        receive_control.error = true;
        *previous_value
      }
    } else {
      *previous_value
    }
  }
}
