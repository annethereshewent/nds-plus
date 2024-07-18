use super::backup_file::BackupFile;

pub struct Flash {
  backup_file: BackupFile
}

impl Flash {
  pub fn new(backup_file: BackupFile) -> Self {
    Self {
      backup_file
    }
  }

  pub fn write(&mut self, data: u8) {

  }

  pub fn read(&self) -> u8 {
    0
  }
}