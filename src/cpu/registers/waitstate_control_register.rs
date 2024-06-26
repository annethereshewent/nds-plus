
pub struct WaitstateControlRegister {
  pub value: u16
}

const FIRST_ACCESS_CYCLES: [u16; 4] = [4,3,2,8];
const WAITSTATE0_SECOND_ACCESS_CYCLES: [u16; 2] = [2,1];
const WAITSTATE1_SECOND_ACCESS_CYCLES: [u16; 2] = [4,1];
const WAITSTATE2_SECOND_ACCESS_CYCLES: [u16; 2] = [8,1];


impl WaitstateControlRegister {
  pub fn new() -> Self {
    Self {
      value: 0
    }
  }

  pub fn sram_wait_ctl_cycles(&self) -> u16 {
    FIRST_ACCESS_CYCLES[(self.value & 0b11) as usize]
  }

  pub fn waitstate_0_first_access_cycles(&self) -> u16 {
    FIRST_ACCESS_CYCLES[((self.value >> 2) & 0b11) as usize]
  }

  pub fn waitstate_0_second_access_cycles(&self) -> u16 {
    WAITSTATE0_SECOND_ACCESS_CYCLES[((self.value >> 4) & 0b1) as usize]
  }

  pub fn waitstate_1_first_access_cycles(&self) -> u16 {
    FIRST_ACCESS_CYCLES[((self.value >> 5) & 0b11) as usize]
  }

  pub fn waitstate_1_second_access_cycles(&self) -> u16 {
    WAITSTATE1_SECOND_ACCESS_CYCLES[((self.value >> 7) & 0b1) as usize]
  }

  pub fn waitstate_2_first_access_cycles(&self) -> u16 {
    FIRST_ACCESS_CYCLES[((self.value >> 8) & 0b11) as usize]
  }

  pub fn waitstate_2_second_access_cycles(&self) -> u16 {
    WAITSTATE2_SECOND_ACCESS_CYCLES[((self.value >> 10) & 0b1) as usize]
  }
}