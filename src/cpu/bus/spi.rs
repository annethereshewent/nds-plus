use super::{backup_file::BackupFile, firmware_data::{FirmwareData, FIRMWARE_CAPACITY}, flash::Flash};

pub struct SPI {
  pub firmware: Flash
}

impl SPI {
  pub fn new(firmware_bytes: Option<BackupFile>) -> Self {
    if let Some(firmware) = firmware_bytes {
      Self {
        firmware: Flash::new(firmware)
      }
    } else {
      Self::hle_direct_boot()
    }
  }

  fn hle_direct_boot() -> Self {
    let mut firmware = Flash::new(BackupFile::new(None, None, FIRMWARE_CAPACITY, false));

    let firmware_data = FirmwareData::new();

    let buffer = &mut firmware.backup_file.buffer;

    buffer[0..0x1d].fill(0);

    buffer[0x2ff] = 0x80;

    firmware_data.header.fill_buffer(buffer);
    firmware_data.user_settings.fill_buffer(buffer);

    Self {
      firmware
    }
  }
}