extern crate ds_emulator;
extern crate console_error_panic_hook;

use ds_emulator::{
  apu::Sample,
  cpu::{
    bus::cartridge::BackupType,
    registers::external_key_input_register::ExternalKeyInputRegister
  },
  gpu::registers::power_control_register1::PowerControlRegister1,
  nds::Nds
};
use wasm_bindgen::prelude::*;
use std::{
  collections::VecDeque,
  panic,
  sync::{
    Arc,
    Mutex
  }
};

#[derive(PartialEq, Eq, Hash)]
#[wasm_bindgen]
pub enum ButtonEvent {
  ButtonA,
  ButtonB,
  ButtonY,
  ButtonX,
  ButtonL,
  ButtonR,
  Select,
  Start,
  Up,
  Down,
  Left,
  Right
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
  ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}


#[wasm_bindgen]
pub struct WasmEmulator {
  nds: Nds
}

#[wasm_bindgen]
impl WasmEmulator {
  #[wasm_bindgen(constructor)]
  pub fn new(
    bios7_bytes: &[u8],
    bios9_bytes: &[u8],
    firmware_bytes: &[u8],
    game_data: &[u8],
  ) -> Self {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let audio_buffer = Arc::new(Mutex::new(VecDeque::new()));

    Self {
      nds: Nds::new(
        None,
        None,
        Some(firmware_bytes.to_vec()),
        bios7_bytes.to_vec(),
        bios9_bytes.to_vec(),
        game_data.to_vec(),
        true,
        audio_buffer
      )
    }
  }

  pub fn touch_screen(&mut self, x: u16, y: u16) {
    console_log!("touching screen at {x},{y}");

    let ref mut bus = *self.nds.bus.borrow_mut();

    bus.touchscreen.touch_screen(x, y);
    bus.arm7.extkeyin.remove(ExternalKeyInputRegister::PEN_DOWN);
  }

  pub fn release_screen(&mut self) {
    let ref mut bus = *self.nds.bus.borrow_mut();

    bus.arm7.extkeyin.insert(ExternalKeyInputRegister::PEN_DOWN);
  }

  pub fn get_game_code(&self) -> u32 {
    self.nds.bus.borrow().cartridge.header.game_code
  }

  pub fn has_saved(&self) -> bool {
    let ref bus = *self.nds.bus.borrow();

    match &bus.cartridge.backup {
      BackupType::None => false,
      BackupType::Eeprom(eeprom) => eeprom.backup_file.has_written,
      BackupType::Flash(flash) => flash.backup_file.has_written,
    }
  }

  pub fn backup_pointer(&self) -> *const u8 {
    let ref bus = *self.nds.bus.borrow();

    match &bus.cartridge.backup {
      BackupType::None => unreachable!(),
      BackupType::Eeprom(eeprom) => eeprom.backup_file.buffer.as_ptr(),
      BackupType::Flash(flash) => flash.backup_file.buffer.as_ptr(),
    }
  }

  pub fn backup_length(&self) -> usize {
    let ref bus = *self.nds.bus.borrow();

    match &bus.cartridge.backup {
      BackupType::None => unreachable!(),
      BackupType::Eeprom(eeprom) => eeprom.backup_file.buffer.len(),
      BackupType::Flash(flash) => flash.backup_file.buffer.len(),
    }
  }

  pub fn set_saved(&mut self, val: bool) {
    let ref mut bus = *self.nds.bus.borrow_mut();

    match &mut bus.cartridge.backup {
      BackupType::None => unreachable!(),
      BackupType::Eeprom(eeprom) => eeprom.backup_file.has_written = val,
      BackupType::Flash(flash) => flash.backup_file.has_written = val,
    }
  }

  pub fn set_backup(&mut self, save_type: String, ram_capacity: usize, bytes: &[u8]) {
    self.nds.bus.borrow_mut().cartridge.set_backup_wasm(bytes, save_type, ram_capacity);
  }

  pub fn update_audio_buffers(&mut self, left_buffer: &mut [f32], right_buffer: &mut [f32]) {
    let ref mut bus = *self.nds.bus.borrow_mut();
    let mut audio_samples = bus.arm7.apu.audio_buffer.lock().unwrap();
    let len = audio_samples.len();

    let mut last_sample = Sample { left: 0.0, right: 0.0 };

    if len > 2 {
      last_sample.left = audio_samples[len - 2];
      last_sample.right = audio_samples[len - 1];
    }

    let mut is_left_sample = true;

    let mut left_index = 0;
    let mut right_index = 0;
    while let Some(sample) = audio_samples.pop_front() {
      if is_left_sample {
        left_buffer[left_index] = sample;
        left_index += 1;
      } else {
        right_buffer[right_index] = sample;
        right_index += 1;
      }
      is_left_sample = !is_left_sample;
    }

  }

  pub fn get_engine_a_picture_pointer(&self) -> *const u8 {
    self.nds.bus.borrow().gpu.engine_a.pixels.as_ptr()
  }

  pub fn get_engine_b_picture_pointer(&self) -> *const u8 {
    self.nds.bus.borrow().gpu.engine_b.pixels.as_ptr()
  }

  pub fn is_top_a(&self) -> bool {
    let ref bus = *self.nds.bus.borrow();

    bus.gpu.powcnt1.contains(PowerControlRegister1::TOP_A)
  }

  pub fn step_frame(&mut self) {
    let mut frame_finished = false;

    while !(frame_finished) {
      frame_finished = self.nds.step();
    }

    let ref mut bus = *self.nds.bus.borrow_mut();

    if bus.scheduler.cycles * 2 >= 0xfff0_0000  {
      let to_subtract = bus.scheduler.rebase_cycles();
      self.nds.arm9_cpu.cycles -= to_subtract * 2;
      self.nds.arm7_cpu.cycles -= to_subtract;
    }

    bus.gpu.frame_finished = false;
  }
}