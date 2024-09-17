use std::{collections::{HashMap, VecDeque}, sync::{Arc, Mutex}};

use ds_emulator::{cpu::registers::{external_key_input_register::ExternalKeyInputRegister, key_input_register::KeyInputRegister}, gpu::registers::power_control_register1::PowerControlRegister1, nds::Nds};

extern crate ds_emulator;

#[swift_bridge::bridge]
mod ffi {
  extern "Rust" {
    type MobileEmulator;

    #[swift_bridge(init)]
    fn new(
      bios7_bytes: &[u8],
      bios9_bytes: &[u8],
      firmware_bytes: &[u8],
      game_data: &[u8],
    ) -> MobileEmulator;

    fn step_frame(&mut self);

    fn get_engine_a_picture_pointer(&self) -> *const u8;
    fn get_engine_b_picture_pointer(&self) -> *const u8;
  }


}

#[derive(PartialEq, Eq, Hash)]
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

pub struct MobileEmulator {
  nds: Nds,
  key_map: HashMap<ButtonEvent, KeyInputRegister>,
  extkey_map: HashMap<ButtonEvent, ExternalKeyInputRegister>
}

impl MobileEmulator {
  pub fn new(
    bios7_bytes: &[u8],
    bios9_bytes: &[u8],
    firmware_bytes: &[u8],
    game_data: &[u8],
  ) -> Self {
    let audio_buffer = Arc::new(Mutex::new(VecDeque::new()));

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
      ),
      key_map,
      extkey_map
    }
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
}