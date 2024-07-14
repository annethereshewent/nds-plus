
#[derive(Copy, Clone)]
pub enum SoundFormat {
  PCM8,
  PCM16,
  IMAADPCM,
  PSG
}

#[derive(Copy, Clone)]
pub struct SoundChannelControlRegister {
  val: u32,
  volume_mul: u32,
  volume_div: u32,
  hold_sample: bool,
  panning: u32,
  wave_duty: u32,
  repeat_mode: u32,
  format: SoundFormat,
  is_started: bool
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
      repeat_mode: 0,
      format: SoundFormat::PCM8,
      is_started: false
    }
  }

  pub fn write(&mut self, value: u32, mask: Option<u32>) {
    let mut val = 0;

    if let Some(mask) = mask {
      val = self.val & mask;
    }

    val |= value;


    self.val = val;

    self.volume_mul = val & 0x7f;
    self.volume_div = (val >> 8) & 0x3;
    self.hold_sample = (val >> 15) & 0b1 == 1;
    self.panning = (val >> 16) & 0x7f;
    self.wave_duty = (val >> 24) & 0x7;
    self.repeat_mode = (val >> 27) & 0x3;
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