use super::{backup_file::BackupFile, flash::Flash};

pub struct SPI {
  pub firmware: Option<Flash>
}

impl SPI {
  pub fn new(firmware_bytes: Option<BackupFile>) -> Self {
    if let Some(firmware) = firmware_bytes {
      Self {
        firmware: Some(Flash::new(firmware))
      }
    } else {
      Self {
        firmware: None
      }
    }
  }
}