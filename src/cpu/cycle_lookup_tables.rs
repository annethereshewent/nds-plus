use super::registers::waitstate_control_register::WaitstateControlRegister;

const LUT_SIZE: usize = 0x100;

const BOARD_RAM_PAGE: usize = 0x2;
const OAM_RAM_PAGE: usize = 0x7;
const VRAM_PAGE: usize = 0x6;
const PALRAM_PAGE: usize = 0x5;

const SRAM_LO_PAGE: usize = 0xe;
const SRAM_HI_PAGE: usize = 0xf;

const WAITSTATE_0_PAGE: usize = 0x8;
const WAITSTATE_1_PAGE: usize = 0xa;
const WAITSTATE_2_PAGE: usize = 0xc;

pub struct CycleLookupTables {
  pub n_cycles_32: [u32; LUT_SIZE],
  pub s_cycles_32: [u32; LUT_SIZE],
  pub n_cycles_16: [u32; LUT_SIZE],
  pub s_cycles_16: [u32; LUT_SIZE]
}

impl CycleLookupTables {
  pub fn new() -> Self {
    Self {
      n_cycles_32: [1; LUT_SIZE],
      s_cycles_32: [1; LUT_SIZE],
      n_cycles_16: [1; LUT_SIZE],
      s_cycles_16: [1; LUT_SIZE]
    }
  }

  pub fn init(&mut self) {
    self.n_cycles_32[BOARD_RAM_PAGE] = 6;
    self.s_cycles_32[BOARD_RAM_PAGE] = 6;
    self.n_cycles_16[BOARD_RAM_PAGE] = 3;
    self.s_cycles_16[BOARD_RAM_PAGE] = 3;

    self.n_cycles_32[OAM_RAM_PAGE] = 2;
    self.s_cycles_32[OAM_RAM_PAGE] = 2;
    self.n_cycles_16[OAM_RAM_PAGE] = 1;
    self.s_cycles_16[OAM_RAM_PAGE] = 1;

    self.n_cycles_32[VRAM_PAGE] = 2;
    self.s_cycles_32[VRAM_PAGE] = 2;
    self.n_cycles_16[VRAM_PAGE] = 1;
    self.s_cycles_16[VRAM_PAGE] = 1;

    self.n_cycles_32[PALRAM_PAGE] = 2;
    self.s_cycles_32[PALRAM_PAGE] = 2;
    self.n_cycles_16[PALRAM_PAGE] = 1;
    self.s_cycles_16[PALRAM_PAGE] = 1;
  }

  pub fn update_tables(&mut self, waitcnt: &WaitstateControlRegister) {
    let sram_wait_cycles = waitcnt.sram_wait_ctl_cycles() as u32;

    self.n_cycles_32[SRAM_LO_PAGE] = sram_wait_cycles;
    self.n_cycles_32[SRAM_LO_PAGE] = sram_wait_cycles;
    self.n_cycles_16[SRAM_HI_PAGE] = sram_wait_cycles;
    self.n_cycles_16[SRAM_HI_PAGE] = sram_wait_cycles;
    self.s_cycles_32[SRAM_LO_PAGE] = sram_wait_cycles;
    self.s_cycles_32[SRAM_LO_PAGE] = sram_wait_cycles;
    self.s_cycles_16[SRAM_HI_PAGE] = sram_wait_cycles;
    self.s_cycles_16[SRAM_HI_PAGE] = sram_wait_cycles;

    for i in 0..2 {
      self.n_cycles_16[WAITSTATE_0_PAGE + i] = 1 + waitcnt.waitstate_0_first_access_cycles() as u32;
      self.s_cycles_16[WAITSTATE_0_PAGE + i] = 1 + waitcnt.waitstate_0_second_access_cycles() as u32;

      self.n_cycles_16[WAITSTATE_1_PAGE + i] = 1 + waitcnt.waitstate_1_first_access_cycles() as u32;
      self.s_cycles_16[WAITSTATE_1_PAGE + i] = 1 + waitcnt.waitstate_1_second_access_cycles() as u32;

      self.n_cycles_16[WAITSTATE_2_PAGE + i] = 1 + waitcnt.waitstate_2_first_access_cycles() as u32;
      self.s_cycles_16[WAITSTATE_2_PAGE + i] = 1 + waitcnt.waitstate_2_second_access_cycles() as u32;

      self.n_cycles_32[WAITSTATE_0_PAGE + i] = self.n_cycles_16[WAITSTATE_0_PAGE + i] + self.s_cycles_16[WAITSTATE_0_PAGE + i];
      self.n_cycles_32[WAITSTATE_1_PAGE + i] = self.n_cycles_16[WAITSTATE_1_PAGE + i] + self.s_cycles_16[WAITSTATE_1_PAGE + i];
      self.n_cycles_32[WAITSTATE_2_PAGE + i] = self.n_cycles_16[WAITSTATE_2_PAGE + i] + self.s_cycles_16[WAITSTATE_2_PAGE + i];

      self.s_cycles_32[WAITSTATE_0_PAGE + i] = 2 * self.s_cycles_16[WAITSTATE_0_PAGE + i];
      self.s_cycles_32[WAITSTATE_1_PAGE + i] = 2 * self.s_cycles_16[WAITSTATE_1_PAGE + i];
      self.s_cycles_32[WAITSTATE_2_PAGE + i] = 2 * self.s_cycles_16[WAITSTATE_2_PAGE + i];
    }
  }
}