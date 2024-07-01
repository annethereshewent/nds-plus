use cp15_control_register::CP15ControlRegister;
use tcm_control_register::TCMControlRegister;

use super::{DTCM_SIZE, ITCM_SIZE};


pub const CP15_INDEX: usize = 15;

pub mod cp15_control_register;
pub mod tcm_control_register;


pub struct CP15 {
  pub control: CP15ControlRegister,
  itcm_control: TCMControlRegister,
  dtcm_control: TCMControlRegister,
  pub arm9_halted: bool
}

impl CP15 {
  pub fn new() -> Self {
    Self {
      control: CP15ControlRegister::from_bits_retain(0x52078),
      itcm_control: TCMControlRegister::new(0x0300000A),
      dtcm_control: TCMControlRegister::new(0x00000020),
      arm9_halted: false
    }
  }

  pub fn read(&self, cn: u32, cm: u32, cp: u32) -> u32 {
    // hardcoded values gotten from another emulator that's fairly accurate, so hopefully these should be ok
    match (cn, cm, cp) {
      (0, 0, 0) => 0x41059461, // Main ID
      (0, 0, 1) => 0x0F0D2112, // Cache type
      (1, 0, 0) => self.control.bits(),
      (9, 1, 0) => self.itcm_control.read(),
      (9, 1, 1) => self.dtcm_control.read(),
      _ => 0
    }
  }

  pub fn write(&mut self, cn: u32, cm: u32, cp: u32, val: u32) {
    match (cn, cm, cp) {
      // control write
      (1, 0, 0) => self.control = CP15ControlRegister::from_bits_retain(val),
      // write cache commands
      (7, 0, 4) if val == 0 => self.arm9_halted = true,
      // write to tcm control registers
      (9, 1, 0) => self.dtcm_control.write(val),
      (9, 1, 1) => self.itcm_control.write(val),
      _ => ()
    }
  }
}