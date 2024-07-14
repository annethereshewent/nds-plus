
#[derive(Copy, Clone)]
pub enum SoundFormat {
  PCM8,
  PCM16,
  IMAADPCM,
  PSG
}

pub struct SoundChannelControlRegister {
  val: u32
}

impl SoundChannelControlRegister {
  pub fn new() -> Self {
    Self {
      val: 0
    }
  }

  pub fn read(&self) -> u32 {
    self.val
  }

  pub fn volume_mul(&self) -> u32 {
    self.val & 0x7f
  }

  pub fn volume_div(&self) -> u32 {
    match self.val >> 8 & 0x3 {
      0 => 1,
      1 => 2,
      2 => 4,
      3 => 16,
      _ => unreachable!()
    }
  }

  pub fn hold_sample(&self) -> bool {
    self.val >> 15 & 0b1 == 1
  }

  pub fn panning(&self) -> u32 {
    (self.val >> 16) & 0x7f
  }

  pub fn wave_duty(&self) -> u32 {
    (self.val >> 24) & 0x7
  }

  pub fn repeat_mode(&self) -> u32 {
    (self.val >> 27) & 0x3
  }

  pub fn format(&self) -> SoundFormat {
    match (self.val >> 29) & 0x3 {
      0 => SoundFormat::PCM8,
      1 => SoundFormat::PCM16,
      2 => SoundFormat::IMAADPCM,
      3 => SoundFormat::PSG,
      _ => unreachable!()
    }
  }
}