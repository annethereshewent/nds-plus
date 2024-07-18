use std::fs::File;

use super::flash::Flash;

pub struct SPI {
  pub firmware: Flash
}

impl SPI {
  pub fn new(firmware_bytes: File) -> Self {
    Self {
      firmware: Flash::new(firmware_bytes)
    }
  }
}