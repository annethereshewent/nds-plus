use std::{collections::VecDeque, sync::{Arc, Mutex}};

use channel::Channel;
use registers::{
  sound_capture_control_register::SoundCaptureControlRegister,
  sound_control_register::{OutputSource, SoundControlRegister}
};

use crate::scheduler::{EventType, Scheduler};

pub mod registers;
pub mod channel;

pub const NUM_SAMPLES: usize = 8192*2;
pub const DS_SAMPLE_RATE: usize = 32768;
pub const INDEX_TABLE: [i32; 8] = [-1,-1,-1,-1,2,4,6,8];
pub const OUT_FREQUENCY: usize = 44100;
pub const CYCLES_PER_SAMPLE: usize = 1024;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BitLength {
  Bit8,
  Bit16,
  Bit32
}

#[derive(Debug, Copy, Clone)]
pub struct Sample<T> {
  pub left: T,
  pub right: T
}

impl Sample<f32> {
  pub fn from(left: i16, right: i16) -> Self {
    Self {
      left: Self::to_f32(left) * 0.5, // samples are way too loud without this
      right: Self::to_f32(right) * 0.5
    }
  }

  fn to_f32(sample: i16) -> f32 {
    if sample < 0 {
      sample as f32 / -(i16::MIN as f32)
    } else {
      sample as f32 / i16::MAX as f32
    }
  }
}


pub struct APU {
  pub soundcnt: SoundControlRegister,
  pub sound_bias: u16,
  pub channels: [Channel; 16],
  pub sndcapcnt: [SoundCaptureControlRegister; 2],
  pub adpcm_table: [u32; 89],
  pub audio_buffer: Arc<Mutex<VecDeque<f32>>>,
  pub previous_value: f32,
  pub phase: f32
}

impl APU {
  pub fn new(scheduler: &mut Scheduler, audio_buffer: Arc<Mutex<VecDeque<f32>>>) -> Self {
    let mut apu = Self {
      soundcnt: SoundControlRegister::new(),
      sound_bias: 0,
      channels: Self::create_channels(),
      sndcapcnt: [SoundCaptureControlRegister::new(); 2],
      adpcm_table: [0; 89],
      audio_buffer,
      previous_value: 0.0,
      phase: 0.0
    };

    scheduler.schedule(
      EventType::GenerateSample,
      CYCLES_PER_SAMPLE
    );

    apu.populate_adpcm_table();

    apu
  }

  fn resample(&mut self, sample: Sample<f32>) {
    while self.phase < 1.0 {
      self.push_sample(sample);

      self.phase += DS_SAMPLE_RATE as f32 / OUT_FREQUENCY as f32;
    }
    self.phase -= 1.0;
  }

  pub fn generate_samples(&mut self, scheduler: &mut Scheduler, cycles_left: usize) {
    scheduler.schedule(EventType::GenerateSample, CYCLES_PER_SAMPLE - cycles_left);

    let mut mixer = Sample { left: 0.0, right: 0.0 };
    let mut ch1 = Sample { left: 0.0, right: 0.0 };
    let mut ch3 = Sample { left: 0.0, right: 0.0 };

    if self.channels[0].soundcnt.is_started || self.channels[0].soundcnt.hold_sample {
      self.channels[0].generate_samples(&mut mixer);
    }

    if self.channels[2].soundcnt.is_started || self.channels[2].soundcnt.hold_sample {
      self.channels[2].generate_samples(&mut mixer);
    }

    for i in 4..self.channels.len() {
      if self.channels[i].soundcnt.is_started || self.channels[i].soundcnt.hold_sample {
        self.channels[i].generate_samples(&mut mixer);
      }
    }

    if self.channels[1].soundcnt.is_started || self.channels[1].soundcnt.hold_sample {
      self.channels[1].generate_samples(&mut ch1);
    }
    if self.channels[3].soundcnt.is_started || self.channels[3].soundcnt.hold_sample {
      self.channels[3].generate_samples(&mut ch3);
    }

    if self.soundcnt.output_ch1_to_mixer {
      mixer.left += ch1.left;
      mixer.right += ch1.right;
    }
    if self.soundcnt.output_ch3_to_mixer {
      mixer.left += ch3.left;
      mixer.right += ch3.right;
    }

    let left_sample = match self.soundcnt.left_output_source {
      OutputSource::Ch1 => ch1.left,
      OutputSource::Mixer => mixer.left,
      OutputSource::Ch3 => ch3.left,
      OutputSource::Ch1and3 => {
        ch1.left + ch3.left
      }
    };

    let right_sample = match self.soundcnt.right_output_source {
      OutputSource::Ch1 => ch1.right,
      OutputSource::Mixer => mixer.right,
      OutputSource::Ch3 => ch3.right,
      OutputSource::Ch1and3 => {
        ch1.right + ch3.right
      }
    };

    let final_sample = Sample { left: self.add_master_volume(left_sample), right: self.add_master_volume(right_sample) };

    self.resample(final_sample);
  }

  pub fn add_master_volume(&self, sample: f32) -> f32 {
    let master_volume = self.soundcnt.master_volume() as f32 / 128.0;
    sample * master_volume
  }

  fn push_sample(&mut self, sample: Sample<f32>) {
    let mut audio_buffer = self.audio_buffer.lock().unwrap();

    if audio_buffer.len() < NUM_SAMPLES {
      audio_buffer.push_back(sample.left);
    }
    if audio_buffer.len() < NUM_SAMPLES {
      audio_buffer.push_back(sample.right);
    }
  }

  fn populate_adpcm_table(&mut self) {
    /*
      =000776d2h, FOR I=0 TO 88, Table[I]=X SHR 16, X=X+(X/10), NEXT I
      Table[3]=000Ah, Table[4]=000Bh, Table[88]=7FFFh, Table[89..127]=0000h
    */
    let mut x: u32 = 0x776d2;
    for i in 0..89 {
      self.adpcm_table[i] = x >> 16;
      x = x + (x/10);
    }
    self.adpcm_table[3] = 0xa;
    self.adpcm_table[4] = 0xb;
    self.adpcm_table[88] = 0x7fff;
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
    let register = if address & !(0b1) & 0xf == 0xa {
      0xa
    } else {
      (address & !(0x3)) & 0xf
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
    let channel_id = (address >> 4) & 0xf;
    // let register = if address & 0xf != 0xa {
    //   (address & !(0x3)) & 0xf
    // } else {
    //   0xa
    // };
    let register = if address & !(0b1) & 0xf == 0xa {
      0xa
    } else {
      (address & !(0x3)) & 0xf
    };

    let value = if bit_length == BitLength::Bit32 {
      val
    } else {
      let old_value = self.read_channels_internal(address);

      match bit_length {
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
        _ => unreachable!()
      }
    };

    match register {
      0x0 => {
        let previous_is_started = self.channels[channel_id as usize].soundcnt.is_started;

        self.channels[channel_id as usize].write_control(value);
        let channel = &mut self.channels[channel_id as usize];

        if !previous_is_started && channel.soundcnt.is_started {
          if channel.timer_value > 0 && channel.loop_start as u32 + channel.sound_length > 0 {
            let time = (0x10000 - channel.timer_value as u32) << 1;
            scheduler.schedule(EventType::StepAudio(channel.id), time as usize);
          }
        } else if !channel.soundcnt.is_started {
          scheduler.remove(EventType::StepAudio(channel_id as usize));
        }
      }
      0x4 => {
        self.channels[channel_id as usize].source_address = value & 0x7ff_ffff;

        self.channels[channel_id as usize].current_address = self.channels[channel_id as usize].source_address;
      }
      0x8 => {
        self.channels[channel_id as usize].write_timer(value as u16, scheduler);

        if bit_length == BitLength::Bit32 {
          self.channels[channel_id as usize].loop_start = (value >> 16) as u16;

          self.channels[channel_id as usize].bytes_left = (self.channels[channel_id as usize].loop_start as u32 + self.channels[channel_id as usize].sound_length) * 4;
        }
      }
      0xa => {
        self.channels[channel_id as usize].loop_start = value as u16;

        self.channels[channel_id as usize].bytes_left = (self.channels[channel_id as usize].loop_start as u32 + self.channels[channel_id as usize].sound_length) * 4;
      }
      0xc => {
        self.channels[channel_id as usize].sound_length = value & 0x3f_ffff;

        self.channels[channel_id as usize].bytes_left = (self.channels[channel_id as usize].sound_length + self.channels[channel_id as usize].sound_length) * 4;
      }
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