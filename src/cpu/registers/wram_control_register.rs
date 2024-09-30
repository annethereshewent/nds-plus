pub struct WRAMControlRegister {
  val: u8,
  pub arm7_size: u32,
  pub arm9_size: u32,
  pub arm7_offset: u32,
  pub arm9_offset: u32
}

impl WRAMControlRegister {
  pub fn new() -> Self {
    let mut wramcnt = Self {
      val: 3, // default value
      arm7_size: 0,
      arm9_size: 0,
      arm7_offset: 0,
      arm9_offset: 0
    };

    wramcnt.update_params();

    wramcnt
  }

  pub fn read(&self) -> u8 {
    self.val
  }

  pub fn write(&mut self, value: u8) {
    self.val = value & 0x3; // only the first two bits matter
    self.update_params();
  }

  fn update_params(&mut self) {
    // (0-3 = 32K/0K, 2nd 16K/1st 16K, 1st 16K/2nd 16K, 0K/32K)
    match self.val {
      0 => {
        self.arm9_size = 2u32.pow(15); // 32 kb
        self.arm7_size = 0;

        self.arm9_offset = 0;
        self.arm7_offset = 0;
      }
      1 => {
        self.arm9_size = 2u32.pow(14);
        self.arm7_size = 2u32.pow(14);


        self.arm9_offset = self.arm9_size;
        self.arm7_offset = 0;
      }
      2 => {
        self.arm9_size = 2u32.pow(14);
        self.arm7_size = 2u32.pow(14);

        self.arm9_offset = 0;
        self.arm7_offset = self.arm7_size;
      }
      3 => {
        self.arm9_size = 0;
        self.arm7_size = 2u32.pow(15);

        self.arm9_offset = 0;
        self.arm7_offset = 0;
      }
      _ => unreachable!("can't happen")
    }
  }
}