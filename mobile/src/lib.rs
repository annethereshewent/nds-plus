use std::{collections::VecDeque, sync::{Arc, Mutex}};

use ds_emulator::{
  apu::Sample, cpu::{bus::{cartridge::BackupType, touchscreen::SAMPLE_SIZE}, registers::{
    external_key_input_register::ExternalKeyInputRegister,
    key_input_register::KeyInputRegister
  }}, gpu::registers::power_control_register1::PowerControlRegister1, nds::Nds
};
use ffi::ButtonEvent;

extern crate ds_emulator;

#[swift_bridge::bridge]
mod ffi {
  enum ButtonEvent {
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
  extern "Rust" {
    type MobileEmulator;

    #[swift_bridge(init)]
    fn new(
      bios7_bytes: &[u8],
      bios9_bytes: &[u8],
      firmware_bytes: &[u8],
      game_data: &[u8],
    ) -> MobileEmulator;

    #[swift_bridge(swift_name = "stepFrame")]
    fn step_frame(&mut self);

    #[swift_bridge(swift_name = "getEngineAPicturePointer")]
    fn get_engine_a_picture_pointer(&self) -> *const u8;

    #[swift_bridge(swift_name = "getEngineBPicturePointer")]
    fn get_engine_b_picture_pointer(&self) -> *const u8;

    #[swift_bridge(swift_name = "isTopA")]
    fn is_top_a(&self) -> bool;

    #[swift_bridge(swift_name = "touchScreen")]
    fn touch_screen(&mut self, x: u16, y: u16);

    #[swift_bridge(swift_name = "releaseScreen")]
    fn release_screen(&mut self);

    #[swift_bridge(swift_name = "updateInput")]
    fn update_input(&mut self, button_event: ButtonEvent, value: bool);

    #[swift_bridge(swift_name = "getGameIconPointer")]
    fn get_game_icon_pointer(&self) -> *const u8;

    #[swift_bridge(swift_name = "getGameCode")]
    fn get_game_code(&self) -> u32;

    #[swift_bridge(swift_name = "setBackup")]
    fn set_backup(&mut self, save_type: String, ram_capacity: usize, bytes: &[u8]);

    #[swift_bridge(swift_name = "backupPointer")]
    fn backup_pointer(&self) -> *const u8;

    #[swift_bridge(swift_name="hasSaved")]
    fn has_saved(&self) -> bool;

    #[swift_bridge(swift_name="setSaved")]
    fn set_saved(&mut self, val: bool);

    #[swift_bridge(swift_name="backupLength")]
    fn backup_length(&self) -> usize;

    #[swift_bridge(swift_name="audioBufferPtr")]
    fn audio_buffer_ptr(&mut self) -> *const f32;

    #[swift_bridge(swift_name="audioBufferLength")]
    fn audio_buffer_length(&self) -> usize;

    #[swift_bridge(swift_name="updateAudioBuffer")]
    fn update_audio_buffer(&mut self, buffer: &[f32]);
  }
}


pub struct MobileEmulator {
  nds: Nds
}

impl MobileEmulator {
  pub fn new(
    bios7_bytes: &[u8],
    bios9_bytes: &[u8],
    firmware_bytes: &[u8],
    game_data: &[u8],
  ) -> Self {
    let audio_buffer = Arc::new(Mutex::new(VecDeque::new()));
    let mic_samples = Arc::new(Mutex::new([0; 2048]));

    Self {
      nds: Nds::new(
        None,
        None,
        Some(firmware_bytes.to_vec()),
        bios7_bytes.to_vec(),
        bios9_bytes.to_vec(),
        game_data.to_vec(),
        true,
        audio_buffer,
        mic_samples.clone()
      )
    }
  }

  pub fn step_frame(&mut self) {
    let mut frame_finished = false;

    let frame_start = self.nds.arm7_cpu.cycles;

    while !(frame_finished) {
      frame_finished = self.nds.step();
      self.nds.bus.borrow_mut().frame_cycles = self.nds.arm7_cpu.cycles - frame_start;
    }

    let ref mut bus = *self.nds.bus.borrow_mut();

    bus.gpu.cap_fps();

    bus.gpu.frame_finished = false;
  }

  pub fn update_input(&mut self, button_event: ButtonEvent, value: bool) {
    let ref mut bus = *self.nds.bus.borrow_mut();
    match button_event {
      // TODO: make KeyInputRegister and ExternalKeyInputRegister naming scheme match
      ButtonEvent::ButtonA => bus.key_input_register.set(KeyInputRegister::ButtonA, !value),
      ButtonEvent::ButtonB => bus.key_input_register.set(KeyInputRegister::ButtonB, !value),
      ButtonEvent::ButtonY => bus.arm7.extkeyin.set(ExternalKeyInputRegister::BUTTON_Y, !value),
      ButtonEvent::ButtonX => bus.arm7.extkeyin.set(ExternalKeyInputRegister::BUTTON_X, !value),
      ButtonEvent::ButtonL => bus.key_input_register.set(KeyInputRegister::ButtonL, !value),
      ButtonEvent::ButtonR => bus.key_input_register.set(KeyInputRegister::ButtonR, !value),
      ButtonEvent::ButtonR3 => (), // TODO implement this
      ButtonEvent::Down => bus.key_input_register.set(KeyInputRegister::Down, !value),
      ButtonEvent::Left => bus.key_input_register.set(KeyInputRegister::Left, !value),
      ButtonEvent::Right => bus.key_input_register.set(KeyInputRegister::Right, !value),
      ButtonEvent::Up => bus.key_input_register.set(KeyInputRegister::Up, !value),
      ButtonEvent::Start => bus.key_input_register.set(KeyInputRegister::Start, !value),
      ButtonEvent::Select => bus.key_input_register.set(KeyInputRegister::Select, !value),
    }
  }

  pub fn get_engine_a_picture_pointer(&self) -> *const u8 {
    self.nds.bus.borrow().gpu.engine_a.pixels.as_ptr()
  }

  pub fn get_engine_b_picture_pointer(&self) -> *const u8 {
    self.nds.bus.borrow().gpu.engine_b.pixels.as_ptr()
  }

  pub fn get_game_icon_pointer(&self) -> *const u8 {
    self.nds.bus.borrow().game_icon.as_ptr()
  }

  pub fn get_game_code(&self) -> u32 {
    self.nds.bus.borrow().cartridge.header.game_code
  }

  pub fn is_top_a(&self) -> bool {
    let ref bus = *self.nds.bus.borrow();

    bus.gpu.powcnt1.contains(PowerControlRegister1::TOP_A)
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

  pub fn audio_buffer_length(&self) -> usize {
    self.nds.bus.borrow().arm7.apu.audio_buffer.lock().unwrap().len()
  }

  pub fn audio_buffer_ptr(&mut self) -> *const f32 {
    let ref mut bus = self.nds.bus.borrow_mut();

    let mut audio_buffer = bus
      .arm7
      .apu
      .audio_buffer
      .lock()
      .unwrap();

    let mut vec = Vec::new();

    for sample in audio_buffer.drain(..) {
      vec.push(sample);
    }

    vec.as_ptr()
  }

  pub fn set_backup(&mut self, save_type: String, ram_capacity: usize, bytes: &[u8]) {
    self.nds.bus.borrow_mut().cartridge.set_backup_external(bytes, save_type, ram_capacity);
  }

  pub fn backup_pointer(&self) -> *const u8 {
    let ref bus = *self.nds.bus.borrow();

    match &bus.cartridge.backup {
      BackupType::None => unreachable!(),
      BackupType::Eeprom(eeprom) => eeprom.backup_file.buffer.as_ptr(),
      BackupType::Flash(flash) => flash.backup_file.buffer.as_ptr(),
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

  pub fn has_saved(&self) -> bool {
    let ref bus = *self.nds.bus.borrow();

    match &bus.cartridge.backup {
      BackupType::None => false,
      BackupType::Eeprom(eeprom) => eeprom.backup_file.has_written,
      BackupType::Flash(flash) => flash.backup_file.has_written,
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

  pub fn update_audio_buffer(&mut self, buffer: &[f32]) {
    if buffer.len() > SAMPLE_SIZE {
      let buffer_i16: Vec<i16> = buffer.iter().map(|sample| Sample::to_i16_single(*sample)).collect();

      self.nds.bus.borrow_mut().touchscreen.update_mic_buffer(&buffer_i16);
    }
  }

}