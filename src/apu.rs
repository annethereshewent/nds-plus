use channel::Channel;
use registers::{
  sound_capture_control_register::SoundCaptureControlRegister,
  sound_control_register::SoundControlRegister
};

use crate::{cpu::CLOCK_RATE, scheduler::{EventType, Scheduler}};

pub mod registers;
pub mod channel;


pub const DS_SAMPLE_RATE: usize = 32768;
pub const INDEX_TABLE: [i32; 8] = [1,-1,-1,-1,2,4,6,8];

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
  pub sndcapcnt: [SoundCaptureControlRegister; 2],
  pub adpcm_table: [u32; 89]
}

impl APU {
  pub fn new(scheduler: &mut Scheduler) -> Self {
    let mut apu = Self {
      soundcnt: SoundControlRegister::new(),
      sound_bias: 0,
      channels: Self::create_channels(),
      sndcapcnt: [SoundCaptureControlRegister::new(); 2],
      adpcm_table: [0; 89]
    };

    let clocks_per_sample = CLOCK_RATE / DS_SAMPLE_RATE;
    scheduler.schedule(
      EventType::GenerateSample,
      clocks_per_sample
    );

    apu.populate_adpcm_table();

    apu
  }

  pub fn generate_samples(&mut self, scheduler: &mut Scheduler) {
    let clocks_per_sample = CLOCK_RATE / DS_SAMPLE_RATE;
    scheduler.schedule(EventType::GenerateSample, clocks_per_sample);
  }

  pub fn populate_adpcm_table(&mut self) {
    /*
      =000776d2h, FOR I=0 TO 88, Table[I]=X SHR 16, X=X+(X/10), NEXT I
      Table[3]=000Ah, Table[4]=000Bh, Table[88]=7FFFh, Table[89..127]=0000h
    */
    let mut x: u32 = 0x776d2;
    for i in 0..89 {
      self.adpcm_table[i] = x >> 16;
      x = x + (x/10);
    }
    self.adpcm_table[3] = 0xa; self.adpcm_table[4] = 0xb; self.adpcm_table[88] = 0x7fff;
  }

  pub fn create_channels() -> [Channel; 16] {
    let mut vec = Vec::with_capacity(16);

    for i in 0..16 {
      vec.push(Channel::new(i));
    }

    vec.try_into().unwrap_or_else(|vec: Vec<Channel>| panic!("expected a vec of length 16 but got a vec of length {}", vec.len()))
  }

  pub fn write_sound_bias(&mut self, value: u16, mask: Option<u16>) {
    let mut val = 0;

    if let Some(mask) = mask {
      val = self.sound_bias & mask;
    }

    val |= value;

    self.sound_bias = val;
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

  pub fn write_channels(&mut self, address: u32, val: u32, scheduler: &mut Scheduler, bit_length: BitLength) {
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

    let channel_id = (address >> 4) & 0xf;
    let register = if address & 0xf != 0xa {
      (address & !(0x3)) & 0xf
    } else {
      0xa
    };

    match register {
      0x0 => {
        let previous_control = self.channels[channel_id as usize].soundcnt;

        self.channels[channel_id as usize].write_control(value);
        let channel = &mut self.channels[channel_id as usize];

        if !previous_control.is_started &&
          channel.soundcnt.is_started &&
          channel.timer_value > 0 &&
          channel.loop_start as u32 + channel.sound_length > 0
        {
          scheduler.schedule(EventType::StepAudio(channel.id), -(channel.timer_value as i16) as u16 as usize);
        } else if !channel.soundcnt.is_started {
          scheduler.remove(EventType::StepAudio(channel_id as usize));
        }
      }
      0x4 => self.channels[channel_id as usize].source_address = value & 0x7ffffff,
      0x8 => {
        self.channels[channel_id as usize].write_timer(value as u16, scheduler);

        if bit_length == BitLength::Bit32 {
          self.channels[channel_id as usize].loop_start = (value >> 16) as u16;
        }
      }
      0xa => self.channels[channel_id as usize].loop_start = value as u16,
      0xc => self.channels[channel_id as usize].sound_length = value & 0x3fffff,
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