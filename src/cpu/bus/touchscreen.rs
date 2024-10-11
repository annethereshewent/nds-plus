use serde::{Deserialize, Serialize};

use crate::gpu::{SCREEN_HEIGHT, SCREEN_WIDTH};

pub const SAMPLE_SIZE: usize = 735;
const CYCLES_PER_FRAME: usize = 560190;

#[derive(Serialize, Deserialize)]
pub struct Touchscreen {
  pub x: u16,
  pub y: u16,

  data: u16,
  return_byte: u8,
  mic_buffer: Box<[i16]>,
  read_pos: usize
}

impl Touchscreen {
  pub fn new() -> Self {
    Self {
      x: 0,
      y: 0,
      data: 0,
      return_byte: 0,
      mic_buffer: vec![0; SAMPLE_SIZE].into_boxed_slice(),
      read_pos: 0
    }
  }

  pub fn write(&mut self, value: u8, frame_cycles: usize) {
    self.return_byte = (self.data >> 8) as u8;

    self.data <<= 8;


    if (value >> 7) & 0b1 == 1 {
      // start bit must be set to write the data
      let channel = (value >> 4) & 0x7;
      self.data = match channel {
        1 => self.y << 3,
        5 => self.x << 3,
        6 => {
          let index = (frame_cycles * SAMPLE_SIZE) / CYCLES_PER_FRAME;

          let sample = if index >= self.mic_buffer.len() {
            self.mic_buffer[self.mic_buffer.len() - 1]
          } else {
            self.mic_buffer[index]
          };

          // sample = if sample > 0x3fff {
          //   0x7fff
          // } else if sample < -0x4000 {
          //   -0x8000
          // } else {
          //   sample << 1
          // };

          (((sample ^-32768) >> 4) as u16) << 3
        },
        _ => 0xfff
      };
    }
  }

  pub fn update_mic_buffer(&mut self, samples: &[i16]) {
    if (self.read_pos + SAMPLE_SIZE) >= samples.len() {
      let len  = samples.len() - self.read_pos;

      let mut buffer_index = 0;

      for i in self.read_pos..len {
        self.mic_buffer[buffer_index] = samples[i];

        buffer_index += 1;
      }

      let diff = SAMPLE_SIZE - len;

      for i in 0..diff {
        self.mic_buffer[buffer_index] = samples[i];
        buffer_index += 1;
      }

      self.read_pos = diff;
    } else {
      self.mic_buffer.copy_from_slice(&samples[self.read_pos..self.read_pos + SAMPLE_SIZE]);

      self.read_pos += SAMPLE_SIZE;
    }
  }

  pub fn deselect(&mut self) {
    self.data = 0;
  }

  pub fn read(&self) -> u8 {
    self.return_byte
  }

  pub fn touch_screen(&mut self, x: u16, y: u16) {
    self.x = x << 4;
    self.y = y << 4;
  }

  pub fn touch_screen_controller(&mut self, x: i16, y: i16) {
    let middle_x = SCREEN_WIDTH  as i16/ 2;
    let middle_y = SCREEN_HEIGHT as i16 / 2;

    let pointer_x = x / 1000;
    let pointer_y = y / 1000;

    self.x = ((middle_x + pointer_x) << 4) as u16;
    self.y = ((middle_y + pointer_y) << 4) as u16;
  }


  pub fn release_screen(&mut self) {
    self.x = 0;
    self.y = 0xfff;
  }
}