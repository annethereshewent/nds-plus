use crate::scheduler::{EventType, Scheduler};

use super::{registers::sound_channel_control_register::{RepeatMode, SoundChannelControlRegister}, Sample, INDEX_TABLE};

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
  pub current_sample: i16,
  pub initial_adpcm_value: i16,
  pub initial_table_index: i32,
  pub adpcm_value: i16,
  pub adpcm_index: i32,
  pub adpcm_lower_bits: bool,
  pub fetching_header: bool
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
      current_sample: 0,
      initial_adpcm_value: 0,
      initial_table_index: 0,
      fetching_header: true,
      adpcm_index: 0,
      adpcm_value: 0,
      adpcm_lower_bits: true
    }
  }

  pub fn generate_samples(&mut self, sample: &mut Sample) {
    sample.left += (self.current_sample as i32 / self.soundcnt.volume_div() as i32) * self.soundcnt.volume_mul() as i32 * (128 - self.soundcnt.panning_factor() as i32);
    sample.right += (self.current_sample as i32 / self.soundcnt.volume_div() as i32) * self.soundcnt.volume_mul() as i32 * self.soundcnt.panning_factor() as i32;
  }

  pub fn set_adpcm_header(&mut self, header: u32) {
    self.initial_adpcm_value = header as u16 as i16;
    self.initial_table_index = ((header >> 16) & 0x7f) as i16 as i32;

    self.adpcm_value = self.initial_adpcm_value;
    self.adpcm_index = self.initial_table_index;

    self.adpcm_index = self.adpcm_index.clamp(0, 88);
  }

  pub fn get_adpcm_sample_address(&mut self, scheduler: &mut Scheduler) -> u32 {
    let mut reset = false;
    let return_address = self.current_address;
    if !self.adpcm_lower_bits {
      self.current_address += 1;
      self.bytes_left -= 1;

      if self.bytes_left == 0 {
        reset = self.handle_end();
      }
    }

    if reset {
      scheduler.schedule(EventType::ResetAudio(self.id), -(self.timer_value as i16) as u16 as usize);
    } else {
      scheduler.schedule(EventType::StepAudio(self.id), -(self.timer_value as i16) as u16 as usize);
    }

    return_address
  }

  pub fn get_adpcm_header_address(&mut self, scheduler: &mut Scheduler) -> u32 {
    let return_address = self.source_address;
    self.bytes_left -= 4;
    self.current_address += 4;

    scheduler.schedule(EventType::StepAudio(self.id), -(self.timer_value as i16) as u16 as usize);

    return_address
  }

  pub fn has_initial_header(&mut self) -> bool {
    let return_val = self.fetching_header;

    self.fetching_header = false;

    return_val
  }

  pub fn get_sample_address(&mut self, byte_width: u32, scheduler: &mut Scheduler) -> u32 {
    let return_address = self.current_address;

    self.bytes_left -= byte_width;

    self.current_address += byte_width;

    let reset = if self.bytes_left == 0 {
      self.handle_end()
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

  pub fn set_adpcm_data(&mut self, byte: u8, adpcm_table: &[u32]) {
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

    let adpcm_table_value = adpcm_table[self.adpcm_index as usize];

    let mut diff = adpcm_table_value / 8;

    let data = if self.adpcm_lower_bits {
      byte & 0xf
    } else {
      byte >> 4
    };

    self.adpcm_lower_bits = !self.adpcm_lower_bits;

    if data & 1 != 0 {
      diff += adpcm_table_value / 4;
    }
    if data & 2 != 0 {
      diff += adpcm_table_value / 2;
    }
    if data & 4 != 0 {
      diff += adpcm_table_value;
    }

    if data & 8 == 0 {
      self.adpcm_value = self.adpcm_value.saturating_add(diff as i16);
    } else {
      self.adpcm_value = self.adpcm_value.saturating_sub(diff as i16);
    }

    self.adpcm_index += INDEX_TABLE[(data as usize) & 0x7];

    self.adpcm_index = self.adpcm_index.clamp(0, 88);

    self.current_sample = self.adpcm_value;
  }

  pub fn reset_audio(&mut self) {
    self.current_sample = 0;
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

    if self.soundcnt.is_started && self.timer_value != 0 && self.sound_length + self.loop_start as u32 != 0 {
      scheduler.schedule(EventType::StepAudio(self.id), -(value as i16) as u16 as usize);
    }
  }
}