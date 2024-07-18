use std::fs::File;

pub struct Flash {
  file: File
}

impl Flash {
  pub fn new(file: File) -> Self {
    Self {
      file
    }
  }

  pub fn write(&mut self, data: u8) {

  }
}