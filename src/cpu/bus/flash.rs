pub struct Flash {
  file: Vec<u8>
}

impl Flash {
  pub fn new(file: Vec<u8>) -> Self {
    Self {
      file
    }
  }
}