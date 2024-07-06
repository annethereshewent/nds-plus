use std::{rc::Rc, cell::Cell};

use crate::{cpu::registers::interrupt_request_register::InterruptRequestRegister, scheduler::{EventType, Scheduler}};

pub const CYCLE_LUT: [u32; 4] = [1, 64, 256, 1024];

#[derive(Clone)]
pub struct Timer {
  pub id: usize,
  pub reload_value: u16,
  pub value: u16,
  pub timer_ctl: TimerControl,
  pub prescalar_frequency: u32,
  pub running: bool,
  is_arm9: bool,
  start_cycles: usize
}

impl Timer {
  pub fn new(id: usize, is_arm9: bool) -> Self {
    Self {
      reload_value: 0,
      value: 0,
      timer_ctl: TimerControl::from_bits_retain(0),
      prescalar_frequency: 0,
      running: false,
      id,
      is_arm9,
      start_cycles: 0
    }
  }

  // pub fn tick(&mut self, cycles: u32) -> bool {
  //   if self.running && !self.timer_ctl.contains(TimerControl::COUNT_UP_TIMING) {
  //     self.cycles += cycles;

  //     let temp = if self.cycles >= self.prescalar_frequency {
  //       let to_add = self.cycles / self.prescalar_frequency;
  //       self.cycles = 0;
  //       self.value.wrapping_add(to_add as u16)
  //     } else {
  //       self.value
  //     };

  //     // timer has overflown
  //     if temp < self.value  {
  //       self.handle_overflow();

  //       return true;
  //     } else {
  //       self.value = temp;
  //     }
  //   }

  //   false
  // }

  pub fn count_up_timer(&mut self, interrupt_request: &mut InterruptRequestRegister) -> bool {
    let mut return_val = false;

    if self.running {
      let temp = self.value.wrapping_add(1);

      // overflow has happened
      if temp < self.value {
        self.handle_overflow(interrupt_request);

        return_val = true;
      } else {
        self.value = temp;
      }
    }

    return_val
  }

  fn handle_overflow(&mut self, interrupt_request: &mut InterruptRequestRegister) {
    self.value = self.reload_value;

    if self.timer_ctl.contains(TimerControl::IRQ_ENABLE) {
      // trigger irq
      interrupt_request.request_timer(self.id);
    }
  }

  pub fn reload_timer_value(&mut self, value: u16) {
    self.reload_value = value;
  }

  pub fn read_timer_value(&self, scheduler: &Scheduler) -> u16 {
    if !self.timer_ctl.contains(TimerControl::COUNT_UP_TIMING) {
      let current_cycles = scheduler.cycles;

      let time_passed = (current_cycles - self.start_cycles) / self.prescalar_frequency as usize;

      return time_passed as u16 + self.value;
    }

    self.value
  }

  pub fn write_timer_control(&mut self, value: u16, scheduler: &mut Scheduler) {
    let new_ctl = TimerControl::from_bits_retain(value);

    self.prescalar_frequency = CYCLE_LUT[new_ctl.prescalar_selection() as usize];

    let event_type = if self.is_arm9 {
      EventType::TIMER9(self.id)
    } else {
      EventType::TIMER7(self.id)
    };

    scheduler.remove(event_type);

    if new_ctl.contains(TimerControl::ENABLED) && !self.timer_ctl.contains(TimerControl::ENABLED) {
      self.value = self.reload_value;
      self.running = true;

      if !self.timer_ctl.contains(TimerControl::COUNT_UP_TIMING) {
        self.start_cycles = scheduler.cycles;
        scheduler.schedule(event_type, self.value as usize * self.prescalar_frequency as usize);
      }
    } else if !new_ctl.contains(TimerControl::ENABLED) {

      self.running = false;
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