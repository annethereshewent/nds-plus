pub struct CartridgeControlRegister {
  pub key1_gap1_length: u32,
  pub key2_encrypt_data: bool,
  pub key2_apply_seed: bool,
  pub key1_gap2_length: u32,
  pub key2_encrypt_command: bool,
  pub data_word_status: bool,
  pub data_block_size: u32,
  pub transfer_clock_rate: bool,
  pub key1_gap_clocks: bool,
  pub release_reset: bool,
  pub data_direction: bool,
  pub block_start_status: bool
}

impl CartridgeControlRegister {
  pub fn new() -> Self {
    Self {
      key1_gap1_length: 0,
      key2_encrypt_data: false,
      key2_apply_seed: false,
      key1_gap2_length: 0,
      key2_encrypt_command: false,
      data_word_status: false,
      data_block_size: 0,
      transfer_clock_rate: false,
      key1_gap_clocks: false,
      release_reset: false,
      data_direction: false,
      block_start_status: false
    }
  }

  /*
      0001A4h - NDS7/NDS9 - ROMCTRL - Gamecard Bus ROMCTRL (R/W)
      0-12  KEY1 gap1 length  (0-1FFFh) (forced min 08F8h by BIOS) (leading gap)
      13    KEY2 encrypt data (0=Disable, 1=Enable KEY2 Encryption for Data)
      14     "SE" Unknown? (usually same as Bit13) (does NOT affect timing?)
      15    KEY2 Apply Seed   (0=No change, 1=Apply Encryption Seed) (Write only)
      16-21 KEY1 gap2 length  (0-3Fh)   (forced min 18h by BIOS) (200h-byte gap)
      22    KEY2 encrypt cmd  (0=Disable, 1=Enable KEY2 Encryption for Commands)
      23    Data-Word Status  (0=Busy, 1=Ready/DRQ) (Read-only)
      24-26 Data Block size   (0=None, 1..6=100h SHL (1..6) bytes, 7=4 bytes)
      27    Transfer CLK rate (0=6.7MHz=33.51MHz/5, 1=4.2MHz=33.51MHz/8)
      28    KEY1 Gap CLKs (0=Hold CLK High during gaps, 1=Output Dummy CLK Pulses)
      29    RESB Release Reset  (0=Reset, 1=Release) (cannot be cleared once set)
      30    Data Direction "WR" (0=Normal/read, 1=Write, for FLASH/NAND carts)
      31    Block Start/Status  (0=Ready, 1=Start/Busy) (IRQ See 40001A0h/Bit14)
   */
  pub fn read(&self, has_access: bool) -> u32 {
    if !has_access {
      return 0;
    }

    self.key1_gap1_length |
      (self.key2_encrypt_data as u32) << 13 |
      self.key1_gap2_length << 16 |
      (self.key2_encrypt_command as u32) << 22 |
      (self.data_word_status as u32) << 23 |
      self.data_block_size << 24 |
      (self.transfer_clock_rate as u32) << 27 |
      (self.key1_gap_clocks as u32) << 28 |
      (self.release_reset as u32) << 29 |
      (self.data_direction as u32) << 30 |
      (self.block_start_status as u32) << 31
  }

  pub fn write(&mut self, val: u32, mask: Option<u32>, has_access: bool) {
    if !has_access {
      return;
    }

    // let value = (self.read(has_access) & mask) | val;

    let mut value = 0;

    if let Some(mask) = mask {
      value &= mask;
    }

    value |= val;

    self.key1_gap1_length = value & 0x1fff;
    self.key2_encrypt_data = (value >> 13) & 0b1 == 1;
    self.key2_apply_seed = (value >> 15) & 0b1 == 1;
    self.key1_gap2_length = (value >> 16) & 0x1f;
    self.key2_encrypt_command = (value >> 22) & 0b1 == 1;
    self.data_block_size = (value >> 24) & 0x7;
    self.transfer_clock_rate = (value >> 27) & 0b1 == 1;
    self.key1_gap_clocks = (value >> 28) & 0b1 == 1;
    self.release_reset = (value >> 29) & 0b1 == 1;
    self.data_direction = (value >> 30) & 0b1 == 1;
    self.block_start_status = (value >> 31) & 0b1 == 1;
  }
}