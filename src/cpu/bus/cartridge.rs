use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fs, ops::Range, path::{Path, PathBuf}};

use cartridge_control_register::CartridgeControlRegister;
use key1_encryption::Key1Encryption;
use spicnt::SPICNT;

use crate::{
  cpu::{
    bus::backup_file::BackupFile,
    dma::dma_channels::DmaChannels,
    registers::interrupt_request_register::InterruptRequestRegister
  },
  scheduler::{
    EventType,
    Scheduler
  },
  util
};

use super::{eeprom::Eeprom, flash::Flash};

pub mod cartridge_control_register;
pub mod spicnt;
pub mod key1_encryption;

pub const CHIP_ID: u32 = 0x1fc2;


#[derive(Serialize, Deserialize)]
pub struct GameInfo {
  game_code: usize,
  rom_size: usize,
  save_type: String,
  ram_capacity: usize
}

pub struct Header {
  game_title: String,
  pub game_code: u32,
  _maker_code: String,
  _unit_code: u8,
  _encryption_seed_select: u8,
  _device_capacity: u8,
  _region: u8,
  _rom_version: u8,
  _autostart: u8,
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
      _maker_code: std::str::from_utf8(&rom[0x10..0x12]).unwrap_or_default().to_string(),
      _unit_code: rom[0x12],
      _encryption_seed_select: rom[0x13],
      _device_capacity: rom[0x14],
      _region: rom[0x1d],
      _rom_version: rom[0x1e],
      _autostart: rom[0x1f],
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

pub enum BackupType {
  None,
  Flash(Flash),
  Eeprom(Eeprom)
}

pub struct Cartridge {
  pub rom: Vec<u8>,
  pub control: CartridgeControlRegister,
  pub spicnt: SPICNT,
  pub header: Header,
  pub command: [u8; 8],
  pub rom_bytes_left: usize,
  pub out_fifo: VecDeque<u32>,
  pub current_word: u32,
  pub key1_encryption: Key1Encryption,
  pub spidata: u8,
  pub backup: BackupType,
  main_area_load: bool
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
      spidata: 0,
      current_word: 0,
      backup: BackupType::None,
      main_area_load: false
    }
  }

  pub fn detect_backup_type(&mut self, save_filename: PathBuf) {
    // thanks to MelonDS for the game db
    let game_db: Vec<GameInfo> = serde_json::from_str(&fs::read_to_string("../game_db.json").unwrap()).unwrap();

    if let Some(entry) = game_db.iter().find(|entry| entry.game_code == self.header.game_code as usize) {
      let backup_file = BackupFile::new(Some(save_filename), None, entry.ram_capacity, false);

      println!("detected backup type {}", entry.save_type);

      self.set_backup(backup_file, entry.ram_capacity, entry.save_type.clone())
    } else {
      println!("warning: game not found in database, resorting to no save");
    }
  }

  pub fn detect_cloud_backup_type(&mut self, bytes: Vec<u8>) {
    let game_db: Vec<GameInfo> = serde_json::from_str(&fs::read_to_string("../game_db.json").unwrap()).unwrap();

    if let Some(entry) = game_db.iter().find(|entry| entry.game_code == self.header.game_code as usize) {
      let backup_file = BackupFile::new(None, Some(bytes), entry.ram_capacity, true);

      println!("detected backup type {}", entry.save_type);

      self.set_backup(backup_file, entry.ram_capacity, entry.save_type.clone())
    } else {
      println!("warning: game not found in database, resorting to no save");
    }
  }

  fn set_backup(&mut self, backup_file: BackupFile, ram_capacity: usize, save_type: String) {
    match save_type.as_str() {
      "eeprom_small" => {
        self.backup = BackupType::Eeprom(Eeprom::new(backup_file, 1));
      }
      "eeprom" => {
        self.backup = BackupType::Eeprom(Eeprom::new(backup_file, 2));
      }
      "eeprom_large" => {
        let address_width = if ram_capacity > 0x1_0000 {
          3
        } else {
          2
        };
        self.backup = BackupType::Eeprom(Eeprom::new(backup_file, address_width));
      }
      "flash" => {
        self.backup = BackupType::Flash(Flash::new(backup_file));
      }
      _ => panic!("backup type not supported: {}", save_type)
    }
  }

  pub fn set_backup_wasm(&mut self, bytes: &[u8], save_type: String, ram_capacity: usize) {
    let backup_file = BackupFile::new(None, Some(bytes.to_vec()), ram_capacity, false);
    self.set_backup(backup_file, ram_capacity, save_type);
  }

  pub fn read_gamecard_bus(&mut self, scheduler: &mut Scheduler, has_access: bool, is_arm9: bool) -> u32 {
    if has_access {
      if self.control.data_word_status {
        self.control.data_word_status = false;

        self.rom_bytes_left -= 4;

        if self.rom_bytes_left > 0 {
          scheduler.schedule(EventType::WordTransfer(is_arm9), self.get_transfer_time() * 4);
        } else {
          // run immediately
          scheduler.schedule(EventType::BlockFinished(is_arm9), 0);
        }
      }

      return self.current_word;
    }

    0
  }

  pub fn write_command(&mut self, command: u8, byte: usize, has_access: bool) {
    if has_access {
      self.command[byte] = command;
    }
  }

  pub fn write_control(&mut self, value: u32, mask: Option<u32>, scheduler: &mut Scheduler, is_arm9: bool, has_access: bool) {
    if has_access {
      self.control.write(value, mask, has_access);

      if (value >> 31) & 0b1 == 1 {
        // run a command
        self.execute_command(scheduler, is_arm9);
      }
    }
  }

  fn execute_command(&mut self, scheduler: &mut Scheduler, is_arm9: bool) {
    self.control.data_word_status = false;
    self.control.block_start_status = true;

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
      scheduler.schedule(EventType::BlockFinished(is_arm9), self.get_transfer_time() * 8);
    } else {
      scheduler.schedule(EventType::WordTransfer(is_arm9), self.get_transfer_time() * 12);
    }
  }

  fn get_transfer_time(&self) -> usize {
    if self.control.transfer_clock_rate {
      8
    } else {
      5
    }
  }

  pub fn on_word_transferred(&mut self, dma: &mut DmaChannels) {
    self.control.data_word_status = true;
    self.current_word = self.out_fifo.pop_front().unwrap();

    dma.notify_cartridge_event();
  }

  pub fn on_block_finished(&mut self, interrupt_request: &mut InterruptRequestRegister) {
    self.control.block_start_status = false;

    if self.spicnt.transfer_ready_irq {
      interrupt_request.insert(InterruptRequestRegister::GAME_CARD_TRANSFER_COMPLETE);
    }

  }

  pub fn write_spidata(&mut self, val: u8, has_access: bool) {
    if has_access {
      match &mut self.backup {
        BackupType::Eeprom(ref mut eeprom) => {
          eeprom.write(val, self.spicnt.hold_chipselect);
        }
        BackupType::Flash(ref mut flash) => {
          flash.write(val, self.spicnt.hold_chipselect);
        }
        BackupType::None => ()
      }
    }
  }

  pub fn read_spidata(&self, has_access: bool) -> u8 {
    if has_access {
      match &self.backup {
        BackupType::Eeprom(ref eeprom) => {
          return eeprom.read();
        }
        BackupType::Flash(ref flash) => {
          return flash.read();
        }
        BackupType::None => return 0
      }
    }

    0
  }

  fn copy_rom(&mut self, range: Range<usize>) {
    for address in range.step_by(4) {
      self.out_fifo.push_back(u32::from_le_bytes(self.rom[address..address+4].try_into().unwrap()));
    }
  }

  fn execute_encrypted_command(&mut self) {
    self.command.reverse();

    let mut command_u32 = self.to_u32();

    self.key1_encryption.decrypt_64bit(&mut command_u32);

    self.command = self.to_u8(&command_u32);

    self.command.reverse();

    let command = self.command[0] >> 4;

    match command {
      0x4 => {
        // Returns 910h dummy bytes
        for _ in 0..self.rom_bytes_left / 4 {
          self.out_fifo.push_back(0xffff_ffff);
        }
      }
      0x1 => {
        for _ in 0..self.rom_bytes_left / 4 {
          self.out_fifo.push_back(CHIP_ID);
        }
      }
      0x2 => {
        let address = ((self.command[2] as usize) & 0xf0) << 8;

        // TODO: possibly move the code below into copy_rom?
        self.copy_rom(address..address+self.rom_bytes_left);

        if address == 0x4000 {
          // encryObj string
          self.out_fifo[0] = 0x72636e65;
          self.out_fifo[1] = 0x6a624f79;

          self.key1_encryption.init_keycode(self.header.game_code, 3, 2);

          for i in (0..512).step_by(2) {
            let mut data = Vec::new();
            data.push(self.out_fifo[i]);
            data.push(self.out_fifo[i+1]);

            self.key1_encryption.encrypt_64bit(&mut data);

            self.out_fifo[i] = data[0];
            self.out_fifo[i+1] = data[1];
          }

          self.key1_encryption.init_keycode(self.header.game_code, 2, 2);

          let mut data = Vec::new();
          data.push(self.out_fifo[0]);
          data.push(self.out_fifo[1]);

          self.key1_encryption.encrypt_64bit(&mut data);

          self.out_fifo[0] = data[0];
          self.out_fifo[1] = data[1];
        }
      }
      0xa => {
        self.key1_encryption.ready = false;
        self.main_area_load = true;

        for _ in 0..self.rom_bytes_left / 4 {
          self.out_fifo.push_back(0);
        }
      }
      _ => {
        println!("unrecognized encrypted command received!");
        for _ in 0..self.rom_bytes_left / 4 {
          self.out_fifo.push_back(0);
        }
      }
    }
  }

  fn to_u32(&self) -> Vec<u32> {
    let mut buffer = Vec::new();
    for i in (0..self.command.len()).step_by(4) {
      let value = u32::from_le_bytes(self.command[i..i+4].try_into().unwrap());
      buffer.push(value);
    }

    buffer
  }

  fn to_u8(&self, command_u32: &[u32]) -> [u8; 8] {
    let mut buffer: [u8; 8] = [0; 8];

    let mut index = 0;

    for i in 0..command_u32.len() {
      let word = command_u32[i];

      buffer[index] = (word & 0xff) as u8;
      buffer[index + 1] = ((word >> 8) & 0xff) as u8;
      buffer[index + 2] = ((word >> 16) & 0xff) as u8;
      buffer[index + 3] = ((word >> 24) & 0xff) as u8;

      index += 4;
    }

    buffer
  }
  fn get_data(&mut self) {
    let mut address = u32::from_be_bytes(self.command[1..5].try_into().unwrap());

    if address < 0x8000 {
      address = 0x8000 + (address & 0x1fff);
    }

    // There is no alignment restriction for the address. However, the datastream
    // wraps to the begin of the current 4K block when address+length crosses a
    // 4K boundary (1000h bytes).
    if address & 0x1000 != (address + self.rom_bytes_left as u32) & 0x1000 {
      let block4k_start = address & !0xfff;
      let block4k_end = block4k_start + 0x1000;
      let leftover = self.rom_bytes_left - (block4k_end as usize - address as usize);
      self.copy_rom(address as usize..block4k_end as usize);
      self.copy_rom(block4k_start as usize..(block4k_start as usize + leftover) as usize);
    } else {
      self.copy_rom((address as usize)..(address as usize + self.rom_bytes_left));
    }
  }

  fn execute_unencrypted_command(&mut self) {
    let command = self.command[0];

    match command {
      0 => {
        // copy header
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
        println!("unhandled command received: {:x}", command);
        for _ in 0..self.rom_bytes_left / 4 {
          self.out_fifo.push_back(0);
        }
      }
    }
  }
}