use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DivisionControlRegister {
  pub val: u16
}

#[derive(PartialEq)]
pub enum DivisionMode {
  Mode0 = 0,
  Mode1 = 1,
  Mode2 = 2
}

impl DivisionControlRegister {
  pub fn new() -> Self {
    Self {
      val: 0
    }
  }

  pub fn write(&mut self, val: u16) {
    self.val = val & 0x3;
  }

  pub fn read(&self) -> u16 {
    self.val
  }

  pub fn mode(&self) -> DivisionMode {
    match self.val & 0x3 {
      0 => DivisionMode::Mode0,
      1 => DivisionMode::Mode1,
      2 => DivisionMode::Mode2,
      3 => DivisionMode::Mode2,
      _ => unreachable!("can't happen")
    }
  }

  pub fn set_division_by_zero(&mut self, val: bool) {
    if val {
      self.val |= 1 << 14;
    } else {
      self.val &= !(1 << 14);
    }
  }

  pub fn division_by_zero(&self) -> bool {
    match self.val >> 14 & 0b1 {
      0 => false,
      1 => true,
      _ => unreachable!("can't happen")
    }
  }

  pub fn is_busy(&self) -> bool {
    self.val >> 15 & 0b1 == 1
  }
}