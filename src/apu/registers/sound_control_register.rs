use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum OutputSource {
  Mixer = 0,
  Ch1 = 1,
  Ch3 = 2,
  Ch1and3 = 3
}

#[derive(Serialize, Deserialize)]
pub struct SoundControlRegister {
  pub master_volume: u16,
  pub val: u16,
  pub left_output_source: OutputSource,
  pub right_output_source: OutputSource,
  pub output_ch1_to_mixer: bool,
  pub output_ch3_to_mixer: bool,
  pub master_enable: bool
}

impl SoundControlRegister {
  pub fn new() -> Self {
    Self {
      master_enable: false,
      master_volume: 0,
      val: 0,
      left_output_source: OutputSource::Mixer,
      right_output_source: OutputSource::Mixer,
      output_ch1_to_mixer: false,
      output_ch3_to_mixer: false
    }
  }

  pub fn read(&self) -> u16 {
    // self.val

    self.master_volume |
    (self.left_output_source as u16) << 8 |
    (self.right_output_source as u16) << 10 |
    (self.output_ch1_to_mixer as u16) << 12 |
    (self.output_ch3_to_mixer as u16) << 13 |
    (self.master_enable as u16) << 15
  }
  pub fn master_volume(&self) -> i32 {
    if self.master_volume == 127 {
      128
    } else {
      self.master_volume as i32
    }
  }

  pub fn write(&mut self, val: u16, mask: Option<u16>) {
    let mut value = 0;

    if let Some(mask) = mask {
      value = self.val & mask;
    }

    value |= val;

    self.val = value;
    self.master_volume = value & 0x7f;
    self.left_output_source = match (value >> 8) & 0x3 {
      0 => OutputSource::Mixer,
      1 => OutputSource::Ch1,
      2 => OutputSource::Ch3,
      3 => OutputSource::Ch1and3,
      _ => unreachable!()
    };

    self.right_output_source = match (value >> 10) & 0x3 {
      0 => OutputSource::Mixer,
      1 => OutputSource::Ch1,
      2 => OutputSource::Ch3,
      3 => OutputSource::Ch1and3,
      _ => unreachable!()
    };

    self.output_ch1_to_mixer = (value >> 12) & 0b1 == 1;
    self.output_ch3_to_mixer = (value >> 13) & 0b1 == 1;
    self.master_enable = (value >> 15) & 0b1 == 1;
  }
}