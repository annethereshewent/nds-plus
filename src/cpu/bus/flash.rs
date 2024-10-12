use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::backup_file::BackupFile;

#[derive(Serialize, Deserialize, Default)]
enum CommandMode {
  #[default]
  AwaitingCommand,
  ProcessingData,
  ReadingRegister
}

#[derive(PartialEq, Serialize, Deserialize, Default)]
enum Command {
  WREN,
  WRDI,
  RDSR,
  READ,
  FAST,
  PW,
  PP,
  PE,
  SE,
  DP,
  RDP,
  #[default]
  None,
  IR
}

impl Command {
  pub fn from(byte: u8) -> Self {
    match byte {
      0x00 | 0x08 => Command::IR,
      0x06 => Command::WREN,
      0x04 => Command::WRDI,
      0x05 => Command::RDSR,
      0x03 => Command::READ,
      0x0b => Command::FAST,
      0x0a => Command::PW,
      0x02 => Command::PP,
      0xdb => Command::PE,
      0xd8 => Command::SE,
      0xb9 => Command::DP,
      0xab => Command::RDP,
      _ => panic!("invalid instruction byte received: {:x}", byte)
    }
  }
}
#[derive(Serialize, Deserialize, Default)]
pub struct Flash {
  pub backup_file: BackupFile,
  write_enable: bool,
  mode: CommandMode,
  address_bytes_left: usize,
  command: Command,
  current_address: u32,
  current_byte: u8,
  write_in_progress: bool
}

impl Flash {
  pub fn new(backup_file: BackupFile) -> Self {
    Self {
      backup_file,
      write_enable: false,
      mode: CommandMode::AwaitingCommand,
      address_bytes_left: 0,
      command: Command::None,
      current_address: 0,
      current_byte: 0,
      write_in_progress: false
    }
  }

  pub fn read_byte(&mut self) {
    self.current_byte = self.backup_file.read(self.current_address as usize);
    self.current_address += 1;
  }

  pub fn write_byte(&mut self, value: u8) {
    self.current_byte = self.backup_file.read(self.current_address as usize);
    self.backup_file.write(self.current_address as usize, value);
    self.current_address += 1;
  }

  pub fn write(&mut self, data: u8, hold: bool) {
    match self.mode {
      CommandMode::AwaitingCommand => {
        self.command = Command::from(data);

        match self.command {
          Command::IR => (),
          Command::WREN => self.write_enable = true,
          Command::WRDI => self.write_enable = false,
          Command::READ => {
            self.address_bytes_left = 3;
            self.current_address = 0;
            self.mode = CommandMode::ProcessingData
          }
          Command::RDSR => {
            self.mode = CommandMode::ReadingRegister;
          }
          Command::PW => {
            if self.write_enable {
              self.address_bytes_left = 3;
              self.current_address = 0;
              self.mode = CommandMode::ProcessingData;
            }
          }
          _ => todo!("not implemented")
        }
      }
      CommandMode::ProcessingData => {
        if self.address_bytes_left > 0 {
          self.current_address = (self.current_address << 8) | data as u32;
          self.address_bytes_left -= 1;
        } else {
          match self.command {
            Command::READ | Command::FAST => self.read_byte(),
            Command::PW => self.write_byte(data),
            _ => unreachable!("shouldn't happen")
          }
        }
      }
      CommandMode::ReadingRegister => {
        self.current_byte = self.write_in_progress as u8 | (self.write_enable as u8) << 1;
      }
    }

    if !hold {
      if self.command == Command::PW {
        self.backup_file.has_written = true;
        if self.backup_file.is_desktop_cloud {
          self.backup_file.last_write = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("an error occurred")
            .as_millis();
        }
      }
       self.mode = CommandMode::AwaitingCommand;
    }
  }

  pub fn deselect(&mut self) {
    self.mode = CommandMode::AwaitingCommand;
  }

  pub fn read(&self) -> u8 {
    self.current_byte
  }
}