use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SquareRootControlRegister {
  pub val: u16
}

#[derive(PartialEq)]
pub enum BitMode {
  Bit32 = 0,
  Bit64 = 1
}

impl SquareRootControlRegister {
  pub fn new() -> Self {
    Self {
      val: 0
    }
  }
  pub fn read(&self) -> u16 {
    self.val
  }

  pub fn write(&mut self, val: u16) {
    self.val = val;
  }

  pub fn mode(&self) -> BitMode {
    match self.val & 1 {
      0 => BitMode::Bit32,
      1 => BitMode::Bit64,
      _ => unreachable!("can't happen")
    }
  }

  pub fn is_busy(&self) -> bool {
    self.val >> 15 & 0b1 == 1
  }
}