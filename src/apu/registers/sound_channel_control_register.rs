
#[derive(Copy, Clone, Debug)]
pub enum SoundFormat {
  PCM8,
  PCM16,
  IMAADPCM,
  PSG
}

#[derive(Copy, Clone, Debug)]
pub enum RepeatMode {
  Manual = 0,
  Loop = 1,
  OneShot = 2
}


#[derive(Copy, Clone, Debug)]
pub struct SoundChannelControlRegister {
  pub val: u32,
  pub volume_mul: u32,
  pub volume_div: u32,
  pub hold_sample: bool,
  pub panning: u32,
  pub wave_duty: u32,
  pub repeat_mode: RepeatMode,
  pub format: SoundFormat,
  pub is_started: bool
}

impl SoundChannelControlRegister {
  pub fn new() -> Self {
    Self {
      val: 0,
      volume_mul: 0,
      volume_div: 0,
      hold_sample: false,
      panning: 0,
      wave_duty: 0,
      repeat_mode: RepeatMode::Manual,
      format: SoundFormat::PCM8,
      is_started: false
    }
  }

  pub fn volume_div(&self) -> u32 {
    match self.volume_div {
      0 => 1,
      1 => 2,
      2 => 4,
      3 => 16,
      _ => unreachable!()
    }
  }

  pub fn panning_factor(&self) -> u32 {
    if self.panning == 127 {
      128
    } else {
      self.panning
    }
  }

  pub fn volume_mul(&self) -> u32 {
    if self.volume_mul == 127 {
      128
    } else {
      self.volume_mul
    }
  }

  pub fn write(&mut self, val: u32) {
    self.val = val;

    self.volume_mul = val & 0x7f;
    self.volume_div = (val >> 8) & 0x3;
    self.hold_sample = (val >> 15) & 0b1 == 1;
    self.panning = (val >> 16) & 0x7f;
    self.wave_duty = (val >> 24) & 0x7;
    self.repeat_mode = match (val >> 27) & 0x3 {
      0 => RepeatMode::Manual,
      1 => RepeatMode::Loop,
      2 => RepeatMode::OneShot,
      _ => panic!("invalid option given for repeat mode")
    };

    self.format = match (val >> 29) & 0x3 {
      0 => SoundFormat::PCM8,
      1 => SoundFormat::PCM16,
      2 => SoundFormat::IMAADPCM,
      3 => SoundFormat::PSG,
      _ => unreachable!()
    };

    self.is_started = (val >> 31) & 0b1 == 1;
  }

  pub fn read(&self) -> u32 {
    self.val
  }
}