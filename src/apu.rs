use channel::Channel;
use registers::{sound_capture_control_register::SoundCaptureControlRegister, sound_control_register::SoundControlRegister};

pub mod registers;
pub mod channel;

#[derive(Copy, Clone, PartialEq)]
pub enum BitLength {
  Bit8,
  Bit16,
  Bit32
}


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
    let register = if address & 0xf != 0xa {
      (address & !(0x3)) & 0xf
    } else {
      0xa
    };

    match register {
      0x0 => self.channels[channel as usize].read_control(),
      0x4 => self.channels[channel as usize].source_address,
      0x8 => self.channels[channel as usize].timer_value as u32,
      0xa => self.channels[channel as usize].loop_start as u32,
      0xc => self.channels[channel as usize].sound_length,
      _ => panic!("channel register not implemented yet: {:X}", register),
    }
  }

  pub fn write_channels(&mut self, address: u32, val: u32, bit_length: BitLength) {
    let value = if bit_length == BitLength::Bit32 {
      val
    } else {
      let old_value = self.read_channels_internal(address);

      match bit_length {
        BitLength::Bit32 => val,
        BitLength::Bit16 => {
          if address & 0x3 == 2 {
            old_value & 0xffff | (val << 16)
          } else {
            old_value & 0xffff0000 | val
          }
        }
        BitLength::Bit8 => {
          match address & 0x3 {
            0 => old_value & 0xffffff00 | val,
            1 => old_value & 0xffff00ff | (val << 8),
            2 => old_value & 0xff00ffff | (val << 16),
            3 => old_value & 0x00ffffff | (val << 24),
            _ => unreachable!()
          }
        }
      }
    };

    let channel = (address >> 4) & 0xf;
    let register = if address & 0xf != 0xa {
      (address & !(0x3)) & 0xf
    } else {
      0xa
    };

    match register {
      0x0 => self.channels[channel as usize].write_control(value),
      0x4 => self.channels[channel as usize].source_address = value & 0x7ffffff,
      0x8 => {
        self.channels[channel as usize].timer_value = value as u16;
        // TODO: schedule something here
      }
      0xa => self.channels[channel as usize].loop_start = value as u16,
      0xc => self.channels[channel as usize].sound_length = value & 0x3fffff,
      _ => panic!("invalid register given for apu write_channels: {:x}", register)
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