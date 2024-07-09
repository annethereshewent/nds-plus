use std::{collections::VecDeque, ops::Range};

use cartridge_control_register::CartridgeControlRegister;
use key1_encryption::Key1Encryption;
use spicnt::SPICNT;

use crate::{scheduler::Scheduler, util};

pub mod cartridge_control_register;
pub mod spicnt;
pub mod key1_encryption;

pub const CHIP_ID: u32 = 0x1fc2;

pub struct Header {
  game_title: String,
  game_code: u32,
  maker_code: String,
  unit_code: u8,
  encryption_seed_select: u8,
  device_capacity: u8,
  region: u8,
  rom_version: u8,
  autostart: u8,
  pub arm9_rom_offset: u32,
  pub arm9_entry_address: u32,
  pub arm9_ram_address: u32,
  pub arm9_size: u32,
  pub arm7_rom_offset: u32,
  pub arm7_entry_address: u32,
  pub arm7_ram_address: u32,
  pub arm7_size: u32
}

impl Header {
  pub fn new(rom: &Vec<u8>) -> Self {
    let header = Self {
      game_title: std::str::from_utf8(&rom[0..0xc]).unwrap_or_default().to_string(),
      game_code: u32::from_le_bytes(rom[0xc..0x10].try_into().unwrap()),
      maker_code: std::str::from_utf8(&rom[0x10..0x12]).unwrap_or_default().to_string(),
      unit_code: rom[0x12],
      encryption_seed_select: rom[0x13],
      device_capacity: rom[0x14],
      region: rom[0x1d],
      rom_version: rom[0x1e],
      autostart: rom[0x1f],
      arm9_rom_offset: util::read_word(rom, 0x20),
      arm9_entry_address: util::read_word(rom, 0x24),
      arm9_ram_address: util::read_word(rom, 0x28),
      arm9_size: util::read_word(rom, 0x2c),
      arm7_rom_offset: util::read_word(rom, 0x30),
      arm7_entry_address: util::read_word(rom, 0x34),
      arm7_ram_address: util::read_word(rom, 0x38),
      arm7_size: util::read_word(rom, 0x3c)
    };

    println!("Game title: {}", header.game_title.trim());

    header
  }
}

pub struct Cartridge {
  pub rom: Vec<u8>,
  pub control: CartridgeControlRegister,
  pub spicnt: SPICNT,
  pub header: Header,
  pub command: [u8; 8],
  pub rom_bytes_left: usize,
  // TODO: maybe change this to an actual byte array instead of a u32
  pub out_fifo: VecDeque<u32>,
  pub key1_encryption: Key1Encryption,
  pub spidata: u8
}

impl Cartridge {
  pub fn new(rom: Vec<u8>, bios7: &[u8]) -> Self {
    Self {
      control: CartridgeControlRegister::new(),
      spicnt: SPICNT::new(),
      header: Header::new(&rom),
      rom,
      command: [0; 8],
      rom_bytes_left: 0,
      out_fifo: VecDeque::new(),
      key1_encryption: Key1Encryption::new(bios7),
      spidata: 0
    }
  }

  pub fn write_command(&mut self, command: u8, byte: usize, has_access: bool) {
    if has_access {
      self.command[byte] = command;
    }
  }

  pub fn write_control(&mut self, value: u32, mask: u32, scheduler: &mut Scheduler, is_arm9: bool, has_access: bool) {
    if has_access {
      self.control.write(value, mask, has_access);

      println!("writing to da control with value {:x}", value);

      if (value >> 31) & 0b1 == 1 {
        // run a command
        println!("executing a command");
        self.execute_command(scheduler, is_arm9);
      }
    }
  }

  fn execute_command(&mut self, scheduler: &mut Scheduler, is_arm9: bool) {
    self.control.data_word_status = false;

    // Data Block size   (0=None, 1..6=100h SHL (1..6) bytes, 7=4 bytes)
    self.rom_bytes_left = match self.control.data_block_size {
      0 => 0,
      7 => 4,
      num => 0x100 << num
    };

    // next check whether to run an encrypted command or unencrypted
    if self.key1_encryption.ready {
      self.execute_encrypted_command();
    } else {
      self.execute_unencrypted_command();
    }

    if self.rom_bytes_left == 0 {
      // schedule event in scheduler TODO
    } else {
      // schedule event in scheduler TODO
    }
  }

  pub fn write_spidata(&mut self, val: u8, has_access: bool) {
    if has_access {
      // TODO
    }
  }

  fn copy_rom(&mut self, range: Range<usize>) {
    for address in range.step_by(4) {
      self.out_fifo.push_back(u32::from_le_bytes(self.rom[address..address+4].try_into().unwrap()));
    }
  }

  fn execute_encrypted_command(&mut self) {
  }

  fn get_data(&mut self) {
    let mut address = u32::from_le_bytes(self.command[1..5].try_into().unwrap());

    if address < 0x8000 {
      address = 0x800 + (address & 0x1ff);
    }
    // There is no alignment restriction for the address. However, the datastream
    // wraps to the begin of the current 4K block when address+length crosses a
    // 4K boundary (1000h bytes).
    if address & 0x1000 != (address + self.rom_bytes_left as u32) & 0x1000 {
      let block4k_start = address & !0xfff;
      let block4k_end = block4k_start + 0x1000;
      let leftover = self.rom_bytes_left - (block4k_end as usize - address as usize);
      self.copy_rom((address as usize..block4k_end as usize));
      self.copy_rom(block4k_start as usize..(block4k_start as usize + leftover) as usize);
    } else {
      self.copy_rom((address as usize)..self.rom_bytes_left);
    }
  }

  fn execute_unencrypted_command(&mut self) {
    let command = self.command[0];

    println!("executing unencrypted command {:x}", command);

    match command {
      0 => {
        // copy header
        println!("copying header, rom_bytes_left = {:x}", self.rom_bytes_left);
        self.copy_rom(0..self.rom_bytes_left);
      }
      0x3c => {
        self.key1_encryption.init_keycode(self.header.game_code, 2, 2);
      }
      0x9f => {
        // dummy read high Z bytes
        for _ in 0..self.rom_bytes_left / 4 {
          self.out_fifo.push_back(0xffff_ffff);
        }
      }
      0x90 => {
        for _ in 0..self.rom_bytes_left / 4 {
          self.out_fifo.push_back(CHIP_ID);
        }
      }
      0xb7 => {
        self.get_data();
      }
      0xb8 => {
        for _ in 0..self.rom_bytes_left / 4 {
          self.out_fifo.push_back(CHIP_ID);
        }
      }
      _ => {
        println!("unhandled command received: {:x}", command)
      }
    }
  }
}