use super::backup_file::BackupFile;

#[derive(Copy, Clone)]
enum WriteProtect {
  None = 0,
  UpperQuarter = 1,
  UpperHalf = 2,
  All = 3
}

#[derive(Copy, Clone)]
enum CommandMode {
  AwaitingCommand,
  ReadingRegister,
  ProcessingData
}

#[derive(Copy, Clone, PartialEq)]
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
  pub fn from(byte: u8, width: usize) -> Self {
    match byte {
      0x6 => Command::WREN,
      0x4 => Command::WRDI,
      0x5 => Command::RDSR,
      0x1 => Command::WRSR,
      0x3 if width < 2 => Command::RDLO,
      0x3 => Command::RD,
      0xb => Command::RDHI,
      0x2 if width < 2 => Command::WRLO,
      0x2 => Command::WR,
      0xa => Command::WRHI,
      _ => panic!("unimplemented command received: {:x}", byte)
    }
  }
}

pub struct Eeprom {
  address_width: usize,
  pub backup_file: BackupFile,
  mode: CommandMode,
  current_address: usize,
  command: Command,
  current_byte: u8,
  write_enabled: bool,
  address_bytes_left: usize,
  write_protect: WriteProtect,
  write_in_progress: bool
}

impl Eeprom {
  pub fn new(backup_file: BackupFile, address_width: usize) -> Self {
    Self {
      address_width,
      backup_file,
      mode: CommandMode::AwaitingCommand,
      current_address: 0,
      command: Command::None,
      current_byte: 0,
      write_enabled: false,
      write_in_progress: false,
      address_bytes_left: 0,
      write_protect: WriteProtect::None
    }
  }


  pub fn read(&self) -> u8 {
    self.current_byte
  }

  pub fn read_data(&mut self) {
    self.current_byte = self.backup_file.read(self.current_address);
    self.current_address += 1;
  }

  pub fn write_data(&mut self, value: u8) {
    self.backup_file.write(self.current_address, value);
    self.current_address += 1;
  }

  pub fn write(&mut self, value: u8, hold: bool) {
    match self.mode {
      CommandMode::AwaitingCommand => {
        if value == 0 {
          return;
        }

        self.command = Command::from(value, self.address_width);

        match self.command {
          Command::WREN => self.write_enabled = true,
          Command::WRDI => self.write_enabled = false,
          Command::RD | Command::RDLO => {
            self.address_bytes_left = self.address_width;
            self.current_address = 0;
            self.mode = CommandMode::ProcessingData;
          }
          Command::RDHI => {
            self.address_bytes_left = self.address_width;
            self.current_address = 1;
            self.mode = CommandMode::ProcessingData;
          }
          Command::WRHI => {
            if self.write_enabled {
              self.address_bytes_left = self.address_width;
              self.current_address = 1; // addresses are in the range from 0x100-0x1ff, the 1 will be shifted 8 bits appropriately
              self.mode = CommandMode::ProcessingData;
            }
          }
          Command::WRLO | Command::WR => {
            if self.write_enabled {
              self.address_bytes_left = self.address_width;
              self.current_address = 0;
              self.mode = CommandMode::ProcessingData;
            }
          }
          Command::WRSR | Command::RDSR => {
            self.mode = CommandMode::ReadingRegister
          }
          Command::None => panic!("shouldn't happen")
        }
      }
      CommandMode::ReadingRegister => {
        match self.command {
          Command::RDSR => {
            self.current_byte = self.write_in_progress as u8 |
              (self.write_enabled as u8) << 1 |
              (self.write_protect as u8) << 2;

            if self.address_width == 1 {
              self.current_byte |= 0xf << 4;
            }

            self.mode = CommandMode::AwaitingCommand;
          }
          Command::WRSR => {
            self.write_protect = match (value >> 2) & 0x3 {
              0 => WriteProtect::None,
              1 => WriteProtect::UpperQuarter,
              2 => WriteProtect::UpperHalf,
              3 => WriteProtect::All,
              _ => unreachable!()
            };

            self.mode = CommandMode::AwaitingCommand;
          }
          _ => unreachable!()
        }
      }
      CommandMode::ProcessingData => {
        if self.address_bytes_left > 0 {
          self.current_address = (self.current_address << 8) | value as usize;
          self.address_bytes_left -= 1;
        } else {
          match self.command {
            Command::RD | Command::RDHI | Command::RDLO => self.read_data(),
            Command::WR | Command::WRLO | Command::WRHI => self.write_data(value),
            _ => unreachable!("shouldn't happen")
          }
        }
      }
    }

    if !hold {
      match self.command {
        Command::WRLO | Command::WRHI | Command::WR => self.backup_file.has_written = true,
        _ => ()
      }

      self.mode = CommandMode::AwaitingCommand;
      self.command = Command::None;
    }

  }
}