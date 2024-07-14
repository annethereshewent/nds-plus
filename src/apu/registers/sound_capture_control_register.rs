#[derive(Copy, Clone)]
pub struct SoundCaptureControlRegister {
  val: u8,
  pub add: bool,
  pub use_channel: bool,
  pub one_shot: bool,
  pub is_pcm8: bool,
  pub is_running: bool
}

impl SoundCaptureControlRegister {
  pub fn new() -> Self {
    Self {
      val: 0,
      add: false,
      use_channel: false,
      one_shot: false,
      is_pcm8: false,
      is_running: false
    }
  }

  pub fn read(&self) -> u8 {
    self.val
  }

  pub fn write(&mut self, val: u8) {
    self.val = val;

    self.add = val & 0b1 == 1;
    self.use_channel = (val >> 1) & 0b1 == 1;
    self.one_shot = (val >> 2) & 0b1 == 1;
    self.is_pcm8 = (val >> 3) & 0b1 == 1;
    self.is_running = (val >> 7) & 0b1 == 1;
  }
}