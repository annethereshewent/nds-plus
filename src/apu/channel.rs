use super::registers::sound_channel_control_register::SoundChannelControlRegister;

#[derive(Copy, Clone)]
pub struct Channel {
  pub soundcnt: SoundChannelControlRegister
}

impl Channel {
  pub fn new() -> Self {
    Self {
      soundcnt: SoundChannelControlRegister::new()
    }
  }

  pub fn read_control(&self) -> u32 {
    self.soundcnt.read()
  }
}