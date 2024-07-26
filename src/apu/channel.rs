use crate::scheduler::{
  EventType,
  Scheduler
};

use super::{
  registers::sound_channel_control_register::{
    RepeatMode,
    SoundChannelControlRegister
  },
  Sample,
  ADPCM_TABLE,
  INDEX_TABLE
};

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
  pub current_sample: f32,
  pub initial_adpcm_value: i16,
  pub initial_table_index: i32,
  pub adpcm_value: i16,
  pub adpcm_index: i32,
  pub pcm_samples_left: usize,
  pub sample_fifo: u32,
  pub noise_lfsr: Option<u16>,
  pub current_psg_value: Option<u32>
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
      current_sample: 0.0,
      initial_adpcm_value: 0,
      initial_table_index: 0,
      adpcm_index: 0,
      adpcm_value: 0,
      sample_fifo: 0,
      pcm_samples_left: 0,
      noise_lfsr: None,
      current_psg_value: None
    }
  }

  pub fn generate_samples(&mut self, sample: &mut Sample<f32>) {
    let volume = (self.soundcnt.volume_mul() as f32 / 128.0) / self.soundcnt.volume_div();
    let panning = self.soundcnt.panning_factor() as f32 / 128.0;

    sample.left += self.current_sample * volume * (1.0 - panning);
    sample.right += self.current_sample * volume * panning;
  }

  pub fn set_adpcm_header(&mut self, header: u32) {
    self.initial_adpcm_value = header as u16 as i16;
    self.initial_table_index = ((header >> 16) & 0x7f) as i16 as i32;

    self.adpcm_value = self.initial_adpcm_value;
    self.adpcm_index = self.initial_table_index;

    self.adpcm_index = self.adpcm_index.clamp(0, 88);
  }

  pub fn get_adpcm_sample_address(&mut self) -> u32 {
    let return_address = self.current_address;

    self.current_address += 4;
    self.bytes_left -= 4;

    return_address
  }

  pub fn schedule(&self, scheduler: &mut Scheduler, should_reset: bool, cycles_left: usize) {
    if self.timer_value != 0 && self.sound_length + self.loop_start as u32 > 0 {
      let time = (0x10000 - self.timer_value as usize) << 1 - cycles_left;
      if should_reset {
        scheduler.schedule(EventType::ResetAudio(self.id), time);
      } else {
        scheduler.schedule(EventType::StepAudio(self.id), time);
      }
    }
  }

  pub fn step_noise(&mut self, scheduler: &mut Scheduler, cycles_left: usize) {
    if let Some(lfsr) = self.noise_lfsr {
      if lfsr & 0b1 == 1 {
        self.noise_lfsr = Some((lfsr >> 1) ^ 0x6000);
        self.current_sample = -1.0;
      } else {
        self.noise_lfsr = Some(lfsr >> 1);
        self.current_sample = 1.0;
      }

      if self.soundcnt.is_started {
        self.schedule(scheduler, false, cycles_left);
      }
    }
  }

  pub fn step_psg(&mut self, scheduler: &mut Scheduler, cycles_left: usize) {
    if let Some(current_value) = self.current_psg_value {
      if current_value ^ 0x7 > self.soundcnt.wave_duty {
        self.current_sample = 1.0;
      } else {
        self.current_sample = -1.0;
      }

      self.current_psg_value = Some((current_value + 1) & 0x7);


      if self.soundcnt.is_started {
        self.schedule(scheduler, false, cycles_left);
      }
    }
  }

  pub fn get_adpcm_header_address(&mut self, scheduler: &mut Scheduler, cycles_left: usize) -> u32 {
    let return_address = self.source_address;
    self.bytes_left -= 4;
    self.current_address += 4;

    self.schedule(scheduler, false, cycles_left);

    return_address
  }

  pub fn has_initial_header(&mut self) -> bool {
    self.current_address == self.source_address
  }

  pub fn get_sample_address(&mut self) -> u32 {
    let return_address = self.current_address;

    self.bytes_left -= 4;

    self.current_address += 4;

    return_address
  }

  pub fn step_sample_8(&mut self, scheduler: &mut Scheduler, cycles_left: usize) {
    // self.current_sample = (sample as i16) << 8;
    self.current_sample = (self.sample_fifo as i8 as f32) / i8::MAX as f32;
    self.sample_fifo >>= 8;
    self.pcm_samples_left -= 1;

    let mut reset = false;

    if self.bytes_left == 0 && self.pcm_samples_left == 0 {
      reset = self.handle_end();
    }

    if self.soundcnt.is_started {
      self.schedule(scheduler, reset, cycles_left);
    }
  }

  pub fn step_sample_16(&mut self, scheduler: &mut Scheduler, cycles_left: usize) {
    self.current_sample = (self.sample_fifo as i16 as f32) / i16::MAX as f32;

    self.sample_fifo >>= 16;
    self.pcm_samples_left -= 1;

    let mut reset = false;

    if self.bytes_left == 0 && self.pcm_samples_left == 0 {
      reset = self.handle_end();
    }

    if self.soundcnt.is_started {
      self.schedule(scheduler, reset, cycles_left);
    }
  }

  pub fn step_adpcm_data(&mut self, scheduler: &mut Scheduler, cycles_left: usize) {
    /*
      per martin korth:
      Diff = AdpcmTable[Index]/8
      IF (data4bit AND 1) THEN Diff = Diff + AdpcmTable[Index]/4
      IF (data4bit AND 2) THEN Diff = Diff + AdpcmTable[Index]/2
      IF (data4bit AND 4) THEN Diff = Diff + AdpcmTable[Index]/1

      IF (Data4bit AND 8)=0 THEN Pcm16bit = Max(Pcm16bit+Diff,+7FFFh)
      IF (Data4bit AND 8)=8 THEN Pcm16bit = Min(Pcm16bit-Diff,-7FFFh)
    And, a note on the second/third lines (with clipping-error):
      Max(+7FFFh) leaves -8000h unclipped (can happen if initial PCM16 was -8000h)
      Min(-7FFFh) clips -8000h to -7FFFh (possibly unlike windows .WAV files?)
     */

    let adpcm_table_value = ADPCM_TABLE[self.adpcm_index as usize];

    let mut diff = adpcm_table_value / 8;
    let mut reset = false;

    let data = self.sample_fifo & 0xf;
    self.sample_fifo >>= 4;
    self.pcm_samples_left -= 1;

    if data & 1 != 0 {
      diff += adpcm_table_value / 4;
    }
    if data & 2 != 0 {
      diff += adpcm_table_value / 2;
    }
    if data & 4 != 0 {
      diff += adpcm_table_value;
    }

    if data & 8 == 8 {
      self.adpcm_value = self.adpcm_value.saturating_sub(diff as i16);
    } else {
      self.adpcm_value = self.adpcm_value.saturating_add(diff as i16);
    }

    self.adpcm_index += INDEX_TABLE[(data as usize) & 0x7];

    self.adpcm_index = self.adpcm_index.clamp(0, 88);

    self.current_sample = self.adpcm_value as f32 / i16::MAX as f32;

    if self.bytes_left == 0 && self.pcm_samples_left == 0 {
      reset = self.handle_end();
    }

    if self.soundcnt.is_started {
      self.schedule(scheduler, reset, cycles_left);
    }
  }

  pub fn reset_audio(&mut self) {
    self.current_sample = 0.0;
    self.soundcnt.is_started = false;
  }

  fn handle_end(&mut self) -> bool {
    match self.soundcnt.repeat_mode {
      RepeatMode::Manual => {
        self.soundcnt.is_started = true;
        true
      }
      RepeatMode::Loop => {
        self.current_address = self.source_address + self.loop_start as u32 * 4;

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

    if self.soundcnt.is_started {
      self.schedule(scheduler, false, 0);
    }
  }
}