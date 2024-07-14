use crate::scheduler::{EventType, Scheduler};

use super::{registers::sound_channel_control_register::{RepeatMode, SoundChannelControlRegister}, Sample};

pub enum ChannelType {
  Normal,
  PSG,
  Noise
}

pub struct Channel {
  pub soundcnt: SoundChannelControlRegister,
  pub source_address: u32,
  pub timer_value: u16,
  pub loop_start: u16,
  pub sound_length: u32,
  pub id: usize,
  pub bytes_left: u32,
  pub current_address: u32,
  pub current_sample: i16
}

impl Channel {
  pub fn new(id: usize) -> Self {
    Self {
      soundcnt: SoundChannelControlRegister::new(),
      source_address: 0,
      timer_value: 0,
      loop_start: 0,
      sound_length: 0,
      id,
      bytes_left: 0,
      current_address: 0,
      current_sample: 0
    }
  }

  pub fn generate_samples(&mut self, sample: &mut Sample) {
    sample.left = (self.current_sample as i32 / self.soundcnt.volume_div() as i32) * self.soundcnt.volume_mul() as i32 * (128 - self.soundcnt.panning_factor() as i32);
    sample.right = (self.current_sample as i32 / self.soundcnt.volume_div() as i32) * self.soundcnt.volume_mul() as i32 * self.soundcnt.panning_factor() as i32;
  }

  pub fn get_sample_address(&mut self, byte_width: u32, scheduler: &mut Scheduler) -> u32 {
    let return_address = self.current_address;

    self.bytes_left -= byte_width;

    self.current_address += byte_width;

    let reset = if self.bytes_left == 0 {
      self.check_audio()
    } else {
      false
    };

    if reset {
      scheduler.schedule(EventType::ResetAudio(self.id), -(self.timer_value as i16) as u16 as usize);
    } else {
      scheduler.schedule(EventType::StepAudio(self.id), -(self.timer_value as i16) as u16 as usize);
    }

    return_address
  }

  pub fn set_sample_8(&mut self, sample: u8) {
    self.current_sample = (sample as i16) << 8;
  }

  pub fn set_sample_16(&mut self, sample: u16) {
    self.current_sample = sample as i16;
  }

  pub fn reset_audio(&mut self) {
    self.current_sample = 0;
    self.soundcnt.is_started = false;
  }

  fn check_audio(&mut self) -> bool {
    match self.soundcnt.repeat_mode {
      RepeatMode::Manual => {
        self.soundcnt.is_started = true;

        true
      }
      RepeatMode::Loop => {
        self.current_address = self.source_address + self.loop_start as u32;
        self.bytes_left = self.sound_length * 4;

        self.soundcnt.is_started = true;

        false
      }
      RepeatMode::OneShot => {
        self.soundcnt.is_started = false;


        true
      }
    }
  }

  pub fn read_control(&self) -> u32 {
    self.soundcnt.read()
  }

  pub fn write_control(&mut self, value: u32) {
    self.soundcnt.write(value);
  }

  pub fn get_channel_type(&self) -> ChannelType {
    let base_range = 0..8;
    let psg_range = 8..14;
    let noise_range = 14..16;

    if base_range.contains(&self.id) {
      return ChannelType::Normal;
    }
    if psg_range.contains(&self.id) {
      return ChannelType::PSG;
    }

    if noise_range.contains(&self.id) {
      return ChannelType::Noise;
    }

    panic!("should not happen");
  }

  pub fn write_timer(&mut self, value: u16, scheduler: &mut Scheduler) {
    self.timer_value = value;

    if self.soundcnt.is_started && self.timer_value != 0 && self.sound_length + self.loop_start as u32 != 0 {
      scheduler.schedule(EventType::StepAudio(self.id), -(value as i16) as u16 as usize);
    }
  }
}