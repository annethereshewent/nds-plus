extern crate ds_emulator;
extern crate console_error_panic_hook;

use ds_emulator::{
  apu::Sample,
  cpu::{
    bus::{backup_file::BackupFile, cartridge::BackupType, spi::SPI, touchscreen::SAMPLE_SIZE},
    registers::{external_key_input_register::ExternalKeyInputRegister, key_input_register::KeyInputRegister}
  },
  gpu::registers::power_control_register1::PowerControlRegister1,
  nds::Nds
};
use wasm_bindgen::prelude::*;
use std::{
  collections::{HashMap, VecDeque},
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
  Right,
  ButtonR3
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
  nds: Nds,
  key_map: HashMap<ButtonEvent, KeyInputRegister>,
  extkey_map: HashMap<ButtonEvent, ExternalKeyInputRegister>,
  state_len: usize
}

#[wasm_bindgen]
impl WasmEmulator {
  #[wasm_bindgen(constructor)]
  pub fn new(
    bios7_bytes: &[u8],
    bios9_bytes: &[u8],
    firmware_bytes: Option<Box<[u8]>>,
    game_data: &[u8],
  ) -> Self {
    // panic::set_hook(Box::new(console_error_panic_hook::hook));

    let audio_buffer = Arc::new(Mutex::new(VecDeque::new()));
    let mic_samples = Arc::new(Mutex::new(vec![0; 2048].into_boxed_slice()));

    let mut key_map = HashMap::new();

    key_map.insert(ButtonEvent::ButtonA, KeyInputRegister::ButtonA);
    key_map.insert(ButtonEvent::ButtonB, KeyInputRegister::ButtonB);
    key_map.insert(ButtonEvent::ButtonL, KeyInputRegister::ButtonL);
    key_map.insert(ButtonEvent::ButtonR, KeyInputRegister::ButtonR);
    key_map.insert(ButtonEvent::Select, KeyInputRegister::Select);
    key_map.insert(ButtonEvent::Start, KeyInputRegister::Start);
    key_map.insert(ButtonEvent::Up, KeyInputRegister::Up);
    key_map.insert(ButtonEvent::Down, KeyInputRegister::Down);
    key_map.insert(ButtonEvent::Left, KeyInputRegister::Left);
    key_map.insert(ButtonEvent::Right, KeyInputRegister::Right);

    let mut extkey_map = HashMap::new();

    extkey_map.insert(ButtonEvent::ButtonY, ExternalKeyInputRegister::BUTTON_Y);
    extkey_map.insert(ButtonEvent::ButtonX, ExternalKeyInputRegister::BUTTON_X);

    let firmware = if let Some(firmware_bytes) = firmware_bytes {
      Some(firmware_bytes.to_vec())
    } else {
      None
    };

    let mut emu = Self {
      nds: Nds::new(
        None,
        firmware,
        bios7_bytes.to_vec(),
        bios9_bytes.to_vec(),
        audio_buffer,
        mic_samples,
      ),
      key_map,
      extkey_map,
      state_len: 0
    };

    emu.nds.init(&game_data.to_vec(), true);

    emu
  }

  pub fn touch_screen(&mut self, x: u16, y: u16) {
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

  pub fn update_input(&mut self, button_event: ButtonEvent, value: bool) {
    let ref mut bus = *self.nds.bus.borrow_mut();
    if let Some(button) = self.key_map.get(&button_event) {
      bus.key_input_register.set(*button, !value);
    } else if let Some(button) = self.extkey_map.get(&button_event) {
      bus.arm7.extkeyin.set(*button, !value)
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
    self.nds.bus.borrow_mut().cartridge.set_backup_external(bytes, save_type, ram_capacity);
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

  pub fn press_screen(&mut self) {
    self.nds.bus.borrow_mut().arm7.extkeyin.remove(ExternalKeyInputRegister::PEN_DOWN);
  }

  pub fn touch_screen_controller(&mut self, x: f32, y: f32) {
    self.nds.bus.borrow_mut().touchscreen.touch_screen_controller(Self::to_i16(x), Self::to_i16(y));
  }

  pub fn update_mic_buffer(&mut self, samples: &[f32]) {
    if samples.len() > SAMPLE_SIZE {
      let samples_i16: Vec<i16> = samples.iter().map(|sample| Sample::to_i16_single(*sample)).collect();

      self.nds.bus.borrow_mut().touchscreen.update_mic_buffer(&samples_i16);
    }
  }

  fn to_i16(value: f32) -> i16 {
    if value >= 0.0 {
      (value * i16::MAX as f32) as i16
    } else {
      (-value * i16::MIN as f32) as i16
    }
  }

  pub fn create_save_state(&mut self) -> *const u8 {
    let buf = self.nds.create_save_state();

    self.state_len = buf.len();

    buf.as_ptr()
  }

  pub fn reload_bios(&mut self, bios7: &[u8], bios9: &[u8]) {
    let ref mut bus = *self.nds.bus.borrow_mut();
    bus.arm7.bios7 = bios7.to_vec();
    bus.arm9.bios9 = bios9.to_vec();
  }

  pub fn reload_firmware(&mut self, firmware: &[u8]) {
    let bytes = firmware.to_vec();

    let ref mut bus = *self.nds.bus.borrow_mut();

    let backup_file = Some(
      BackupFile::new(
        None,
        Some(bytes),
        firmware.len(),
        false
      )
    );

    bus.spi = SPI::new(backup_file);
  }

  pub fn hle_firmware(&mut self) {
    let ref mut bus = self.nds.bus.borrow_mut();

    bus.spi = SPI::new(None);
  }

  pub fn reload_rom(&mut self, rom: &[u8]) {
    let ref mut bus = *self.nds.bus.borrow_mut();

    bus.cartridge.rom = rom.to_vec();
  }

  pub fn save_state_length(&self) -> usize {
    self.state_len
  }

  pub fn set_pause(&mut self, val: bool) {
    if val {
      self.nds.paused = true;
    } else {
      self.nds.paused = false;
    }
  }

  pub fn load_save_state(&mut self, data: &[u8]) {
    self.nds.load_save_state(&data);

    {
      let ref mut bus = *self.nds.bus.borrow_mut();

      // recreate mic and audio buffers
      bus.touchscreen.mic_buffer = vec![0; SAMPLE_SIZE].into_boxed_slice();

      let audio_buffer = Arc::new(Mutex::new(VecDeque::new()));

      bus.arm7.apu.audio_buffer = audio_buffer;
    }

    self.nds.arm7_cpu.bus = self.nds.bus.clone();
    self.nds.arm9_cpu.bus = self.nds.bus.clone();

    // repopulate arm and thumb luts
    self.nds.arm7_cpu.populate_arm_lut();
    self.nds.arm9_cpu.populate_arm_lut();

    self.nds.arm7_cpu.populate_thumb_lut();
    self.nds.arm9_cpu.populate_thumb_lut();
  }

  pub fn step_frame(&mut self) {
    let mut frame_finished = false;

    let start_cycles = self.nds.arm7_cpu.cycles;

    while !(frame_finished) {
      if !self.nds.paused {
        frame_finished = self.nds.step();
        self.nds.bus.borrow_mut().frame_cycles = self.nds.arm7_cpu.cycles - start_cycles;
      } else {
        break;
      }
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