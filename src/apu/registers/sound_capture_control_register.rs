pub struct SoundCaptureControlRegister {
  val: u8,
  pub add: bool,
  pub use_channel: bool,
  pub one_shot: bool,
  pub is_pcm8: bool,
  pub is_running: bool,
  pub destination_address: u32,
  pub current_address: u32,
  pub capture_length: u16,
  pub bytes_left: u16,
  pub timer_value: u16,
  pub fifo: [u8; 32],
  pub fifo_pos: u8,
  pub read_half: bool
}

impl SoundCaptureControlRegister {
  pub fn new() -> Self {
    Self {
      val: 0,
      add: false,
      use_channel: false,
      one_shot: false,
      is_pcm8: false,
      is_running: false,
      destination_address: 0,
      current_address: 0,
      capture_length: 0,
      bytes_left: 0,
      timer_value: 0,
      fifo: [0; 32],
      fifo_pos: 0,
      read_half: false
    }
  }

  pub fn read(&self) -> u8 {
    self.val
  }

  pub fn write_destination(&mut self, val: u32, mask: Option<u32>) {
    let mut value = 0;

    if let Some(mask) = mask {
      value = self.destination_address & mask;
    }

    value |= val;

    self.destination_address = value & 0x7ff_ffff;
    self.current_address = self.destination_address;
  }

  pub fn write_length(&mut self, val: u16, mask: Option<u16>) {
    let mut value = 0;

    if let Some(mask) = mask {
      value = self.capture_length & mask;
    }

    value |= val;

    self.capture_length = value;

    self.bytes_left = self.capture_length * 4;
  }

  pub fn write(&mut self, val: u8) {
    let previous_running = self.is_running;
    self.val = val;

    self.add = val & 0b1 == 1;
    self.use_channel = (val >> 1) & 0b1 == 1;
    self.one_shot = (val >> 2) & 0b1 == 1;
    self.is_pcm8 = (val >> 3) & 0b1 == 1;
    self.is_running = (val >> 7) & 0b1 == 1;

    if !previous_running && self.is_running {
      self.read_half = false;
    }
  }
}