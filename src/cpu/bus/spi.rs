use super::{backup_file::BackupFile, firmware_data::{FirmwareData, FIRMWARE_CAPACITY}, flash::Flash};

pub struct SPI {
  pub firmware: Flash,
  firmware_data: Option<FirmwareData>
}

impl SPI {
  pub fn new(firmware_bytes: Option<BackupFile>) -> Self {
    if let Some(firmware) = firmware_bytes {
      Self {
        firmware: Flash::new(firmware),
        firmware_data: None
      }
    } else {
      Self::hle_direct_boot()
    }
  }

  fn hle_direct_boot() -> Self {
    let mut firmware = Flash::new(BackupFile::new(None, None, FIRMWARE_CAPACITY, false));
    Self {
      firmware_data: Some(FirmwareData::new(&mut firmware.backup_file.buffer)),
      firmware
    }
  }
}