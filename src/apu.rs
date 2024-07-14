use channel::Channel;
use registers::{sound_capture_control_register::SoundCaptureControlRegister, sound_control_register::SoundControlRegister};

pub mod registers;
pub mod channel;

pub struct APU {
  pub soundcnt: SoundControlRegister,
  pub sound_bias: u16,
  pub channels: [Channel; 16],
  pub sndcapcnt: [SoundCaptureControlRegister; 2]
}

impl APU {
  pub fn new() -> Self {
    Self {
      soundcnt: SoundControlRegister::new(),
      sound_bias: 0,
      channels: [Channel::new(); 16],
      sndcapcnt: [SoundCaptureControlRegister::new(); 2]
    }
  }

  pub fn read_channels(&self, address: u32) -> u16 {
    let channel = (address >> 4) & 0xf;
    let register = (address & !(0x3)) & 0xf;

    let value = if register == 0 {
      self.channels[channel as usize].read_control()
    } else {
      0
    };

    if address & 0x3 == 2 {
      (value >> 16) as u16
    } else {
      value as u16
    }
  }
}