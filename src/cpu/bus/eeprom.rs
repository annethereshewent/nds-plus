use std::fs::File;

pub struct Eeprom {
  pub is_small: bool,
  pub backup_file: File
}

impl Eeprom {
  pub fn new(backup_file: File, is_small: bool) -> Self {
    Self {
      is_small,
      backup_file
    }
  }
  pub fn write(&mut self, value: u8) {

  }
}