use super::registers::sound_channel_control_register::SoundChannelControlRegister;

#[derive(Copy, Clone)]
pub struct Channel {
  pub soundcnt: SoundChannelControlRegister,
  pub source_address: u32,
  pub timer_value: u16,
  pub loop_start: u16,
  pub sound_length: u32
}

impl Channel {
  pub fn new() -> Self {
    Self {
      soundcnt: SoundChannelControlRegister::new(),
      source_address: 0,
      timer_value: 0,
      loop_start: 0,
      sound_length: 0
    }
  }

  pub fn read_control(&self) -> u32 {
    self.soundcnt.read()
  }

  pub fn write_control(&mut self, value: u32) {
    self.soundcnt.write(value);
  }
}