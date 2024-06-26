use std::{rc::Rc, cell::Cell};

use crate::cpu::registers::interrupt_request_register::InterruptRequestRegister;

pub const CYCLE_LUT: [u32; 4] = [1, 64, 256, 1024];

#[derive(Clone)]
pub struct Timer {
  pub id: usize,
  pub reload_value: u16,
  pub value: u16,
  pub timer_ctl: TimerControl,
  pub prescalar_frequency: u32,
  pub running: bool,
  pub cycles: u32,
  interrupt_request: Rc<Cell<InterruptRequestRegister>>
}

impl Timer {
  pub fn new(id: usize, interrupt_request: Rc<Cell<InterruptRequestRegister>>) -> Self {
    Self {
      reload_value: 0,
      value: 0,
      timer_ctl: TimerControl::from_bits_retain(0),
      prescalar_frequency: 0,
      running: false,
      cycles: 0,
      id,
      interrupt_request
    }
  }

  pub fn tick(&mut self, cycles: u32) -> bool {
    if self.running && !self.timer_ctl.contains(TimerControl::COUNT_UP_TIMING) {
      self.cycles += cycles;

      let temp = if self.cycles >= self.prescalar_frequency {
        let to_add = self.cycles / self.prescalar_frequency;
        self.cycles = 0;
        self.value.wrapping_add(to_add as u16)
      } else {
        self.value
      };

      // timer has overflown
      if temp < self.value  {
        self.handle_overflow();

        return true;
      } else {
        self.value = temp;
      }
    }

    false
  }

  pub fn count_up_timer(&mut self) -> bool {
    let mut return_val = false;

    if self.running {
      let temp = self.value.wrapping_add(1);

      // overflow has happened
      if temp < self.value {
        self.handle_overflow();

        return_val = true;
      } else {
        self.value = temp;
      }
    }

    return_val
  }

  fn handle_overflow(&mut self) {
    self.value = self.reload_value;
    self.cycles = 0;

    if self.timer_ctl.contains(TimerControl::IRQ_ENABLE) {
      // trigger irq
      let mut interrupt_request = self.interrupt_request.get();

      interrupt_request.request_timer(self.id);

      self.interrupt_request.set(interrupt_request);
    }
  }

  pub fn reload_timer_value(&mut self, value: u16) {
    self.reload_value = value;
  }

  pub fn write_timer_control(&mut self, value: u16) {
    let new_ctl = TimerControl::from_bits_retain(value);

    self.prescalar_frequency = CYCLE_LUT[new_ctl.prescalar_selection() as usize];

    if new_ctl.contains(TimerControl::ENABLED) && !self.timer_ctl.contains(TimerControl::ENABLED) {
      self.value = self.reload_value;
      self.cycles = 0;
      self.running = true;
    } else if !new_ctl.contains(TimerControl::ENABLED) {
      self.running = false;
      self.cycles = 0;
    }

    self.timer_ctl = new_ctl;
  }
}

bitflags! {
  #[derive(Copy, Clone)]
  pub struct TimerControl: u16 {
    const COUNT_UP_TIMING = 0b1 << 2;
    const IRQ_ENABLE = 0b1 << 6;
    const ENABLED = 0b1 << 7;
  }
}

impl TimerControl {
  pub fn prescalar_selection(&self) -> u16 {
    self.bits() & 0b11
  }
}