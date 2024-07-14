use crate::scheduler::{EventType, Scheduler};

use super::registers::sound_channel_control_register::SoundChannelControlRegister;

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
  pub current_address: u32
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
      current_address: 0
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