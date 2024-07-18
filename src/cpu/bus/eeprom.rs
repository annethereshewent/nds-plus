use std::fs::File;

use super::backup_file::BackupFile;

#[derive(Copy, Clone)]
enum CommandMode {
  AwaitingCommand,
  HandlingCommand,
  AwaitingAddressLo,
  AwaitingAddressHi,
  ReadingData,
  WritingData
}

#[derive(Copy, Clone)]
enum Command {
  WREN,
  WRDI,
  RDSR,
  WRSR,
  RD,
  WR,
  RDLO,
  RDHI,
  WRLO,
  WRHI,
  None
}

impl Command {
  pub fn from(byte: u8) -> Self {
    match byte {
      0x6 => Command::WREN,
      0x4 => Command::WRDI,
      0x5 => Command::RDSR,
      0x1 => Command::WRSR,
      _ => panic!("unimplemented command received: {:x}", byte)
    }
  }
}

pub struct Eeprom {
  pub is_small: bool,
  pub backup_file: BackupFile,
  mode: CommandMode,
  current_address: usize,
  command: Command,
  current_byte: u8,
  write_enabled: bool
}

impl Eeprom {
  pub fn new(backup_file: BackupFile, is_small: bool) -> Self {
    Self {
      is_small,
      backup_file,
      mode: CommandMode::AwaitingCommand,
      current_address: 0,
      command: Command::None,
      current_byte: 0,
      write_enabled: false
    }
  }


  pub fn read(&self) -> u8 {
    self.current_byte
  }

  pub fn write(&mut self, value: u8, hold: bool) {
    match self.mode {
      CommandMode::AwaitingCommand => {
        self.command = Command::from(value);
        self.mode = CommandMode::HandlingCommand;
      }
      CommandMode::HandlingCommand => {
        match self.command {
          Command::RD => {
            self.mode = CommandMode::AwaitingAddressLo;
          }
          Command::WREN => {
            self.write_enabled = true;
            self.mode = CommandMode::AwaitingCommand;
          }
          Command::WRDI => {
            self.write_enabled = false;
            self.mode = CommandMode::AwaitingCommand;
          }
          Command::WR => {
            if self.write_enabled {
              self.mode = CommandMode::AwaitingAddressLo;
            }
          }
          Command::WRHI => {
            if self.write_enabled {

            }
          }
          Command::WRLO => {

          }
          _ => todo!()
        }
      }
      CommandMode::AwaitingAddressLo => {
        self.current_address = value as usize;
        self.mode = CommandMode::AwaitingAddressHi;
      }
      CommandMode::AwaitingAddressHi => {
        self.current_address = (self.current_address << 8) | value as usize;
        self.mode = match self.command {
          Command::RD | Command::RDHI | Command::RDLO => CommandMode::ReadingData,
          Command::WR | Command::WRLO | Command::WRHI => CommandMode::WritingData,
          _ => todo!()
        }
      }
      CommandMode::ReadingData => {
        self.current_byte = self.backup_file.read(self.current_address);
        self.current_address += 1;
      }
      CommandMode::WritingData => {
        self.backup_file.write(self.current_address, value);
        self.current_address += 1;
      }
    }

    if !hold {
      self.mode = CommandMode::AwaitingCommand;
    }

  }
}