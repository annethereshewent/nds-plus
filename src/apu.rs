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

  fn read_channels_internal(&self, address: u32) -> u32 {
    let channel = (address >> 4) & 0xf;
    let register = (address & !(0x3)) & 0xf;

    match register {
      0x0 => self.channels[channel as usize].read_control(),
      0x4 => self.channels[channel as usize].source_address,
      0x8 => self.channels[channel as usize].timer_value,
      _ => panic!("channel register not implemented yet: {:X}", register),
      // 0x8 => 0, // self.channels[channel as usize].timer_value as u32
      // 0xa => 0, // self.channels[channel as usize].l() as u32,
      // 0xc => 0, // self.channels[channel as usize].
    }
  }

  pub fn write_channels(&mut self, address: u32, val: u32, use_mask: bool) {
    let mut value = 0;

    if use_mask {
      let mask = if address & 0x3 == 2 {
        0xffff
      } else {
        0xffff0000
      };

      value = self.read_channels_internal(address) & mask;
    }

    value |= val;


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