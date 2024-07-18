use std::fs::File;

use super::{backup_file::BackupFile, flash::Flash};

pub struct SPI {
  pub firmware: Flash
}

impl SPI {
  pub fn new(firmware_bytes: BackupFile) -> Self {
    Self {
      firmware: Flash::new(firmware_bytes)
    }
  }
}