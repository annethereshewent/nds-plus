use std::{
  collections::VecDeque,
  fs,
  path::PathBuf,
  sync::{
    Arc,
    Mutex
  }
};

use crate::{apu::Sample, gpu::color::Color, number::Number};
use backup_file::BackupFile;
use cartridge::{
  Cartridge,
  CHIP_ID
};
use cp15::CP15;
use num_integer::Roots;
use serde::{Deserialize, Serialize};
use spi::SPI;
use touchscreen::Touchscreen;

use crate::{
  apu::{
    channel::ChannelType,
    registers::sound_channel_control_register::SoundFormat,
    APU
  },
  gpu::GPU,
  scheduler::Scheduler
};

use super::{
  dma::{
    dma_channel::{
      registers::dma_control_register::DmaControlRegister,
      DmaParams
    },
    dma_channels::DmaChannels
  },
  registers::{
    division_control_register::{
      DivisionControlRegister,
      DivisionMode
    },
    external_key_input_register::ExternalKeyInputRegister,
    external_memory::{
      AccessRights,
      ExternalMemory
    },
    interrupt_enable_register::InterruptEnableRegister,
    interrupt_request_register::InterruptRequestRegister,
    ipc_fifo_control_register::{
      IPCFifoControlRegister,
      FIFO_CAPACITY
    },
    ipc_sync_register::IPCSyncRegister,
    key_input_register::KeyInputRegister,
    real_time_clock_register::RealTimeClockRegister,
    spi_control_register::{
      DeviceSelect,
      SPIControlRegister
    },
    square_root_control_register::{
      BitMode,
      SquareRootControlRegister
    },
    wram_control_register::WRAMControlRegister
  },
  timers::Timers,
  MemoryAccess
};

pub mod arm7;
pub mod arm9;
pub mod cp15;
pub mod spi;
pub mod flash;
pub mod cartridge;
pub mod touchscreen;
pub mod eeprom;
pub mod backup_file;
pub mod firmware_data;

pub const ITCM_SIZE: usize = 0x8000;
pub const DTCM_SIZE: usize = 0x4000;
const MAIN_MEMORY_SIZE: usize = 0x40_0000;
const WRAM_SIZE: usize = 0x1_0000;
const SHARED_WRAM_SIZE: usize = 0x8000;


#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum HaltMode {
  None = 0,
  GbaMode = 1,
  Halt = 2,
  Sleep = 3
}

#[derive(Serialize, Deserialize)]
pub struct Arm9Bus {
  pub timers: Timers,
  pub dma: DmaChannels,
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
  sqrt_result: u32,
  pub dma_fill: [u32; 4]
}

#[derive(Serialize, Deserialize)]
pub struct Arm7Bus {
  pub timers: Timers,
  pub dma: DmaChannels,
  pub bios7: Vec<u8>,
  pub wram: Box<[u8]>,
  pub postflg: bool,
  pub interrupt_master_enable: bool,
  pub ipcsync: IPCSyncRegister,
  pub ipcfifocnt: IPCFifoControlRegister,
  pub interrupt_request: InterruptRequestRegister,
  pub interrupt_enable: InterruptEnableRegister,
  pub spicnt: SPIControlRegister,
  pub extkeyin: ExternalKeyInputRegister,
  pub haltcnt: HaltMode,
  pub apu: APU,
  pub rtc: RealTimeClockRegister
}

#[derive(Serialize, Deserialize)]
pub struct Bus {
  pub arm9: Arm9Bus,
  pub arm7: Arm7Bus,
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
  exmem: ExternalMemory,
  pub touchscreen: Touchscreen,
  pub debug_on: bool,
  pub game_icon: Box<[u8]>,
  pub frame_cycles: usize
}

impl Bus {
  pub fn new(
     firmware_path: Option<PathBuf>,
     firmware_bytes: Option<Vec<u8>>,
     bios7_bytes: Vec<u8>,
     bios9_bytes: Vec<u8>,
     audio_buffer: Arc<Mutex<VecDeque<f32>>>
  ) -> Self {
    let dma_channels7 = DmaChannels::new(false);
    let dma_channels9 = DmaChannels::new(true);

    let mut scheduler = Scheduler::new();


    let backup_file = if firmware_path.is_some() {
      match fs::metadata(&firmware_path.as_ref().unwrap()) {
        Ok(metadata) => {
          Some(BackupFile::new(firmware_path, firmware_bytes, metadata.len() as usize, false))
        }
        Err(_) => None
      }
    } else if firmware_bytes.is_some() {
      let capacity = firmware_bytes.as_ref().unwrap().len();
      Some(BackupFile::new(firmware_path, firmware_bytes, capacity, false))
    } else {
      None
    };

    Self {
      arm9: Arm9Bus {
        timers: Timers::new(true),
        bios9: bios9_bytes,
        dma: dma_channels9,
        cp15: CP15::new(),
        postflg: false,
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
        div_remainder: 0,
        dma_fill: [0; 4],
      },
      shared_wram: vec![0; SHARED_WRAM_SIZE].into_boxed_slice(),
      main_memory: vec![0; MAIN_MEMORY_SIZE].into_boxed_slice(),
      itcm: vec![0; ITCM_SIZE].into_boxed_slice(),
      dtcm: vec![0; DTCM_SIZE].into_boxed_slice(),
      spi: SPI::new(backup_file),
      cartridge: Cartridge::new(&bios7_bytes),
      wramcnt: WRAMControlRegister::new(),
      gpu: GPU::new(&mut scheduler),
      key_input_register: KeyInputRegister::from_bits_truncate(0x3ff),
      exmem: ExternalMemory::new(),
      touchscreen: Touchscreen::new(),
      frame_cycles: 0,
      arm7: Arm7Bus {
        timers: Timers::new(false),
        bios7: bios7_bytes,
        dma: dma_channels7,
        wram: vec![0; WRAM_SIZE].into_boxed_slice(),
        postflg: false,
        interrupt_master_enable: false,
        ipcsync: IPCSyncRegister::new(),
        ipcfifocnt: IPCFifoControlRegister::new(),
        interrupt_request: InterruptRequestRegister::from_bits_retain(0),
        interrupt_enable: InterruptEnableRegister::from_bits_retain(0),
        spicnt: SPIControlRegister::new(),
        extkeyin: ExternalKeyInputRegister::new(),
        haltcnt: HaltMode::None,
        apu: APU::new(&mut scheduler, audio_buffer),
        rtc: RealTimeClockRegister::new()
      },
      scheduler,
      debug_on: false,
      game_icon: vec![0; 32 * 32 * 4].into_boxed_slice()
    }
  }

  pub fn reset(&mut self) -> Self {
    let firmware_bytes = self.spi.firmware.backup_file.reset();

    let mut scheduler = Scheduler::new();
    Self {
      arm9: Arm9Bus {
        timers: Timers::new(true),
        bios9: self.arm9.bios9.clone(),
        dma: DmaChannels::new(true),
        cp15: CP15::new(),
        postflg: true,
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
        div_remainder: 0,
        dma_fill: [0; 4],
      },
      shared_wram: vec![0; SHARED_WRAM_SIZE].into_boxed_slice(),
      main_memory: vec![0; MAIN_MEMORY_SIZE].into_boxed_slice(),
      itcm: vec![0; ITCM_SIZE].into_boxed_slice(),
      dtcm: vec![0; DTCM_SIZE].into_boxed_slice(),
      spi: SPI::new(Some(firmware_bytes)),
      cartridge: Cartridge::new(&self.arm7.bios7),
      wramcnt: WRAMControlRegister::new(),
      gpu: GPU::new(&mut scheduler),
      key_input_register: KeyInputRegister::from_bits_truncate(0x3ff),
      exmem: ExternalMemory::new(),
      touchscreen: Touchscreen::new(),
      arm7: Arm7Bus {
        timers: Timers::new(false),
        bios7: self.arm7.bios7.clone(),
        dma: DmaChannels::new(false),
        wram: vec![0; WRAM_SIZE].into_boxed_slice(),
        postflg: true,
        interrupt_master_enable: false,
        ipcsync: IPCSyncRegister::new(),
        ipcfifocnt: IPCFifoControlRegister::new(),
        interrupt_request: InterruptRequestRegister::from_bits_retain(0),
        interrupt_enable: InterruptEnableRegister::from_bits_retain(0),
        spicnt: SPIControlRegister::new(),
        extkeyin: ExternalKeyInputRegister::new(),
        haltcnt: HaltMode::None,
        apu: APU::new(&mut scheduler, self.arm7.apu.audio_buffer.clone()),
        rtc: RealTimeClockRegister::new()
      },
      scheduler,
      debug_on: false,
      game_icon: vec![0; 32 * 32 * 4].into_boxed_slice(),
      frame_cycles: 0
    }
  }

  pub fn is_halted(&self, is_arm9: bool) -> bool {
    if is_arm9 {
      self.arm9.cp15.arm9_halted
    } else {
      self.arm7.haltcnt == HaltMode::Halt
    }
  }

  fn handle_dma(&mut self, params: &mut DmaParams, is_arm9: bool) -> u32 {
    let mut access = MemoryAccess::NonSequential;
    let mut cpu_cycles = 0;
    if params.fifo_mode {
      for _ in 0..4 {
        let (value, cycles) = self.load_32(params.source_address & !(0b11), access, is_arm9);

        cpu_cycles += cycles;

        cpu_cycles += self.store_32(params.destination_address & !(0b11), value, access, is_arm9);

        access = MemoryAccess::Sequential;
        params.source_address += 4;
      }
    } else if params.word_size == 4 {
      for _ in 0..params.count {
        let (word, cycles) = self.load_32(params.source_address & !(0b11), access, is_arm9);

        cpu_cycles += cycles;

        cpu_cycles += self.store_32(params.destination_address & !(0b11), word, access, is_arm9);

        access = MemoryAccess::Sequential;
        params.source_address = (params.source_address as i32).wrapping_add(params.source_adjust) as u32;
        params.destination_address = (params.destination_address as i32).wrapping_add(params.destination_adjust) as u32;
      }
    } else {
      for _ in 0..params.count {
        let (half_word, cycles) = self.load_16(params.source_address & !(0b1), access, is_arm9);

        cpu_cycles += cycles;

        cpu_cycles += self.store_16(params.destination_address & !(0b1), half_word, access, is_arm9);
        access = MemoryAccess::Sequential;
        params.source_address = (params.source_address as i32).wrapping_add(params.source_adjust) as u32;
        params.destination_address = (params.destination_address as i32).wrapping_add(params.destination_adjust) as u32;
      }
    }

    // 2 idle cycles
    cpu_cycles += 2;


    cpu_cycles
  }

  pub fn check_dma(&mut self, is_arm9: bool) -> u32 {
    let mut cpu_cycles = 0;

    if is_arm9 && self.arm9.dma.has_pending_transfers() {
      for i in 0..4 {
        if let Some(mut params) = self.arm9.dma.get_transfer_parameters(i) {
          cpu_cycles += self.handle_dma(&mut params, is_arm9);

          // i would DRY this code up by adding it to the handle DMA method, but Rust is being a jerk
          // about ownership :/
          let channel = &mut self.arm9.dma.channels[i];

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
    } else if !is_arm9 && self.arm7.dma.has_pending_transfers() {
      for i in 0..4 {
        if let Some(mut params) = self.arm7.dma.get_transfer_parameters(i) {
          cpu_cycles += self.handle_dma(&mut params, is_arm9);

          // update internal destination and source address for the dma channel as well.
          // see above comment
          let channel = &mut self.arm7.dma.channels[i];

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
  fn load_32(&mut self, address: u32, _access: MemoryAccess, is_arm9: bool) -> (u32, u32) {
    // TODO: write this method
    // self.get_cycles(address, access, MemoryWidth::Width32);

    let cpu_cycles = 1;

    if !is_arm9 {
      (self.arm7_mem_read_32(address), cpu_cycles)
    } else {
      (self.arm9_mem_read_32(address), cpu_cycles)
    }
  }

 fn load_16(&mut self, address: u32, _access: MemoryAccess, is_arm9: bool) -> (u16, u32) {
    // TODO: write this method
    // self.get_cycles(address, access, MemoryWidth::Width16);

    let cpu_cycles = 1;

    if !is_arm9 {
      (self.arm7_mem_read_16(address), cpu_cycles)
    } else {
      (self.arm9_mem_read_16(address), cpu_cycles)
    }
  }

  fn store_16(&mut self, address: u32, value: u16, _access: MemoryAccess, is_arm9: bool) -> u32 {
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

  fn store_32(&mut self, address: u32, value: u32, _access: MemoryAccess, is_arm9: bool) -> u32 {
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

  pub fn write_haltcnt(&mut self, value: u8) {
    self.arm7.haltcnt = match (value >> 6) & 0x3 {
      0 => HaltMode::None,
      1 => HaltMode::GbaMode,
      2 => HaltMode::Halt,
      3 => HaltMode::Sleep,
      _ => unreachable!()
    };
  }

  fn load_icon(&mut self) {
    let offset = unsafe { *(&self.cartridge.rom[0x68] as *const u8 as *const u32) };

    let icon_base = (0x20 + offset) as usize;
    let icon_bytes = &self.cartridge.rom[icon_base..icon_base + 0x200];

    let palette_base = 0x220 + offset as usize;

    let mut tiles = [Color::new(); 32 * 32];

    let icon_palette = &self.cartridge.rom[palette_base..palette_base + 0x20];
    for i in 0..32 * 32 {
      let palette_index = if i & 0b1 == 0 {
        icon_bytes[i >> 1] & 0xf
      } else {
        (icon_bytes[i >> 1] >> 4) & 0xf
      };

      let color_raw = if palette_index == 0 {
        0xffff
      } else {
        unsafe { *(&icon_palette[(palette_index * 2) as usize] as *const u8 as *const u16 )}
      };

      let r = color_raw & 0x1f;
      let g = (color_raw >> 5) & 0x1f;
      let b = (color_raw >> 10) & 0x1f;

      let color = Color {
        r: r as u8,
        g: g as u8,
        b: b as u8,
        alpha: None
      }.convert();

      tiles[i] = color;
    }

    // TODO: refactor this
    for i in 0..4 {
      for j in 0..8 {
        for k in 0..4 {
          for x_pos_in_tile in 0..8 {
            let icon_index = (256 * i + 32 * j + 8 * k + x_pos_in_tile) * 4;
            let tile_index = 256 * i + 8 * j + 64 * k + x_pos_in_tile;

            self.game_icon[icon_index] = tiles[tile_index].r;
            self.game_icon[icon_index + 1] = tiles[tile_index].g;
            self.game_icon[icon_index + 2] = tiles[tile_index].b;
            self.game_icon[icon_index + 3] = 0xff;
          }
        }
      }
    }
  }

  pub fn skip_bios(&mut self) {
    // load header into RAM starting at address 0x27ffe00 (per the docs)
    let address = 0x27ffe00 & (MAIN_MEMORY_SIZE - 1);

    self.arm7.postflg = true;
    self.arm9.postflg = true;

    self.main_memory[address..address + 0x170].copy_from_slice(&self.cartridge.rom[0..0x170]);

    let arm9_rom_address = self.cartridge.header.arm9_rom_offset;
    let arm9_ram_address = self.cartridge.header.arm9_ram_address;
    let arm9_size = self.cartridge.header.arm9_size;

    self.load_icon();

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

  pub fn write_spi_data(&mut self, value: u8) {
    if self.arm7.spicnt.spi_bus_enabled {
      match self.arm7.spicnt.device {
        DeviceSelect::Touchscreen => self.touchscreen.write(value, self.frame_cycles),
        DeviceSelect::Firmware => self.spi.firmware.write(value, self.arm7.spicnt.chipselect_hold),
        _ => ()
      }
    }
  }

  pub fn read_spi_data(&self) -> u8 {
    if self.arm7.spicnt.spi_bus_enabled {
      return match self.arm7.spicnt.device {
        DeviceSelect::Firmware => self.spi.firmware.read(),
        DeviceSelect::Touchscreen => self.touchscreen.read(),
        _ => 0
      }
    }
    0
  }

  pub fn read_gba_rom<T: Number>(&self, address: u32, is_arm9: bool) -> T {
    let exmemcnt = if is_arm9 {
      &self.exmem.arm9_exmem
    } else {
      &self.exmem.arm7_exmem
    };

    if is_arm9 && self.exmem.gba_access_rights == AccessRights::Arm9 || !is_arm9 && self.exmem.gba_access_rights == AccessRights::Arm7 {
      // return garbage values depending on exmem properties
      let value = match exmemcnt.gba_rom_1st_access {
        0 => (address / 2) | 0xfe08, // 10 clocks
        1 => address / 2, // 8 clocks
        2 => address / 2, // 6 clocks
        3 => 0xffff,
        _ => unreachable!()
      } & 0xffff;

      let value = match address & 0x3 {
        0 => value,
        1 => value >> 8,
        2 => 0,
        3 => 0,
        _ => unreachable!()
      } as u16;

      // for some reason the unwrap fails sometimes, so as a hack
      // i'm returning 0 for now, which seems to work ok
      return num::cast::<u16, T>(value).unwrap_or(num::zero());
    }

    // return back 0 for the deselected cpu
    num::zero()
  }

  pub fn step_audio(&mut self, channel_id: usize, cycles_left: usize) {
    match self.arm7.apu.channels[channel_id].soundcnt.format {
      SoundFormat::PCM8 => {
        if self.arm7.apu.channels[channel_id].pcm_samples_left == 0 {
          let sample_address = self.arm7.apu.channels[channel_id].get_sample_address();

          let word = self.arm7_mem_read_32(sample_address);
          self.arm7.apu.channels[channel_id].sample_fifo = word;
          self.arm7.apu.channels[channel_id].pcm_samples_left = 4;
        }

        self.arm7.apu.channels[channel_id].step_sample_8(&mut self.scheduler, cycles_left);
      }
      SoundFormat::PCM16 => {
        if self.arm7.apu.channels[channel_id].pcm_samples_left == 0 {
          let sample_address = self.arm7.apu.channels[channel_id].get_sample_address();

          let word = self.arm7_mem_read_32(sample_address);

          self.arm7.apu.channels[channel_id].sample_fifo = word;
          self.arm7.apu.channels[channel_id].pcm_samples_left = 2;
        }

        self.arm7.apu.channels[channel_id].step_sample_16(&mut self.scheduler, cycles_left);
      }
      SoundFormat::IMAADPCM => {
        if self.arm7.apu.channels[channel_id].has_initial_header() {
          let header_address = self.arm7.apu.channels[channel_id].get_adpcm_header_address();

          let header = self.arm7_mem_read_32(header_address);

          self.arm7.apu.channels[channel_id].set_adpcm_header(header);
        }
        if self.arm7.apu.channels[channel_id].pcm_samples_left == 0 {
          let sample_address = self.arm7.apu.channels[channel_id].get_sample_address();

          let word = self.arm7_mem_read_32(sample_address);

          self.arm7.apu.channels[channel_id].sample_fifo = word;

          self.arm7.apu.channels[channel_id].pcm_samples_left = 8;
        }

        self.arm7.apu.channels[channel_id].step_adpcm_data(&mut self.scheduler, cycles_left);
      }
      SoundFormat::PSG => {
        // initialize the channel
        if self.arm7.apu.channels[channel_id].noise_lfsr.is_none() {
          self.arm7.apu.channels[channel_id].noise_lfsr = Some(0x7fff);
          self.arm7.apu.channels[channel_id].current_psg_value = Some(0);
        }

        match self.arm7.apu.channels[channel_id].get_channel_type() {
          ChannelType::Noise => self.arm7.apu.channels[channel_id].step_noise(&mut self.scheduler, cycles_left),
          ChannelType::PSG => self.arm7.apu.channels[channel_id].step_psg(&mut self.scheduler, cycles_left),
          ChannelType::Normal => println!("warning: using a normal channel for psg samples")
        }
      }
    }

    match channel_id {
      1 => self.capture_audio(0),
      3 => self.capture_audio(1),
      _ => ()
    }
  }

  pub fn capture_audio(&mut self, capture_id: usize) {
    if !self.arm7.apu.sndcapcnt[capture_id].is_running {
      return;
    }
    let mut mixer = Sample::<f32>::new();
    if !self.arm7.apu.sndcapcnt[capture_id].use_channel {
      self.arm7.apu.generate_mixer(&mut mixer);
    }

    // rust once again not letting me do something simple like let sndcapcnt = &mut self.arm7.apu.sndcapcnt so i have to type that out every single time
    let (capture_sample, add_sample, mixer_out) = match capture_id {
      0 => (self.arm7.apu.channels[0].apply_volume(), self.arm7.apu.channels[1].apply_volume(), mixer.to_i16().left),
      1 => (self.arm7.apu.channels[2].apply_volume(), self.arm7.apu.channels[3].apply_volume(), mixer.to_i16().right),
      _ => unreachable!()
    };

    let mut sample = if self.arm7.apu.sndcapcnt[capture_id].use_channel {
      if self.arm7.apu.sndcapcnt[capture_id].add {
        Sample::to_i16_single(capture_sample + add_sample)
      } else if capture_sample < 0.0 && add_sample < 0.0 {
        -0x8000
      } else {
        Sample::to_i16_single(capture_sample)
      }
    } else {
      mixer_out
    };

    if self.arm7.apu.sndcapcnt[capture_id].is_pcm8 {
      sample >>= 8;

      self.arm7.apu.sndcapcnt[capture_id].fifo[self.arm7.apu.sndcapcnt[capture_id].fifo_pos as usize] = sample as u8;

      self.arm7.apu.sndcapcnt[capture_id].bytes_left -= 1;

      self.arm7.apu.sndcapcnt[capture_id].fifo_pos = (self.arm7.apu.sndcapcnt[capture_id].fifo_pos + 1) & 0x1f;
    } else {
      let fifo_pos = self.arm7.apu.sndcapcnt[capture_id].fifo_pos & !0b1;
      unsafe { *(&mut self.arm7.apu.sndcapcnt[capture_id].fifo[fifo_pos as usize] as *mut u8 as *mut i16) = sample };

      self.arm7.apu.sndcapcnt[capture_id].bytes_left -= 2;

      self.arm7.apu.sndcapcnt[capture_id].fifo_pos = (self.arm7.apu.sndcapcnt[capture_id].fifo_pos + 2) & 0x1e;
    }

    let offset = if self.arm7.apu.sndcapcnt[capture_id].read_half {
      16
    } else {
      0
    };

    let fifo_pos = self.arm7.apu.sndcapcnt[capture_id].fifo_pos;

    if self.arm7.apu.sndcapcnt[capture_id].bytes_left == 0 {
      self.flush_fifo(capture_id);

      self.arm7.apu.sndcapcnt[capture_id].fifo_pos = 0;

      if self.arm7.apu.sndcapcnt[capture_id].one_shot {
        self.arm7.apu.sndcapcnt[capture_id].is_running = false;
        self.arm7.apu.sndcapcnt[capture_id].add = false;
      } else {
        self.arm7.apu.sndcapcnt[capture_id].bytes_left = self.arm7.apu.sndcapcnt[capture_id].capture_length * 4;

        self.arm7.apu.sndcapcnt[capture_id].current_address = self.arm7.apu.sndcapcnt[capture_id].destination_address;

        self.arm7.apu.sndcapcnt[capture_id].read_half = false;
      }
    } else if (fifo_pos - offset) & 0x1f >= 16 {
      self.flush_fifo(capture_id);
    }
  }

  fn flush_fifo(&mut self, capture_id: usize) {
    let base = if self.arm7.apu.sndcapcnt[capture_id].read_half {
      16
    } else {
      0
    };

    for i in (base..base + 16).step_by(4) {
      let value = unsafe { *(&self.arm7.apu.sndcapcnt[capture_id].fifo[i] as *const u8 as *const u32) };

      self.arm7_mem_write(self.arm7.apu.sndcapcnt[capture_id].current_address, value);

      self.arm7.apu.sndcapcnt[capture_id].current_address += 4;

      let destination_address = self.arm7.apu.sndcapcnt[capture_id].destination_address;
      let destination_end = destination_address + self.arm7.apu.sndcapcnt[capture_id].capture_length as u32 * 4;

      if self.arm7.apu.sndcapcnt[capture_id].current_address >= destination_end {
        break;
      }
    }

    self.arm7.apu.sndcapcnt[capture_id].read_half = !self.arm7.apu.sndcapcnt[capture_id].read_half;
  }


  fn write_spicnt(&mut self, value: u16) {
    let previous_enable = self.arm7.spicnt.spi_bus_enabled;
    let previous_device = self.arm7.spicnt.device;

    self.arm7.spicnt.write(value);

    if previous_enable && !self.arm7.spicnt.spi_bus_enabled {
      match previous_device {
        DeviceSelect::Firmware => self.spi.firmware.deselect(),
        DeviceSelect::Touchscreen => self.touchscreen.deselect(),
        _ => ()
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
      if receive_control.enabled && receive_control.receive_not_empty_irq && send_control.fifo.is_empty() {
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
        if send_control.enabled && send_control.send_empty_irq && receive_control.fifo.is_empty() {
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
