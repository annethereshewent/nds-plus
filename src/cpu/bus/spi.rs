use super::flash::Flash;

pub struct SPI {
  pub firmware: Flash
}

impl SPI {
  pub fn new(firmware_bytes: Vec<u8>) -> Self {
    Self {
      firmware: Flash::new(firmware_bytes)
    }
  }
}