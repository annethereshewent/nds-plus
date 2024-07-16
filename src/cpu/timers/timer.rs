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

  pub fn count_up_timer(&mut self, interrupt_request: &mut InterruptRequestRegister, scheduler: &mut Scheduler, cycles_left: usize) -> bool {
    let mut return_val = false;

    if self.running {
      let temp = self.value.wrapping_add(1);

      // overflow has happened
      if temp < self.value {
        self.handle_overflow(interrupt_request, scheduler, cycles_left);

        return_val = true;
      } else {
        self.value = temp;
      }
    }

    return_val
  }

  pub fn handle_overflow(&mut self, interrupt_request: &mut InterruptRequestRegister, scheduler: &mut Scheduler, cycles_left: usize) {
    self.value = self.reload_value;

    let event_type = if self.is_arm9 {
      EventType::Timer9(self.id)
    } else {
      EventType::Timer7(self.id)
    };

    let cycles_till_overflow = self.prescalar_frequency * (0x1_0000 - self.value as u32);

    scheduler.schedule(event_type, cycles_till_overflow as usize - cycles_left);
    self.start_cycles = scheduler.cycles;

    if self.timer_ctl.contains(TimerControl::IRQ_ENABLE) {
      // trigger irq
      interrupt_request.request_timer(self.id);
    }
  }

  pub fn reload_timer_value(&mut self, value: u16) {
    self.reload_value = value;
  }

  pub fn read_timer_value(&self, scheduler: &Scheduler) -> u16 {
    if !self.timer_ctl.contains(TimerControl::COUNT_UP_TIMING) && self.timer_ctl.contains(TimerControl::ENABLED) {
      let current_cycles = scheduler.cycles;

      let prescalar = if self.prescalar_frequency != 0 {
        self.prescalar_frequency
      } else {
        CYCLE_LUT[self.timer_ctl.prescalar_selection() as usize]
      };

      let time_passed = (current_cycles - self.start_cycles) / prescalar as usize;

      return time_passed as u16 + self.value;
    }

    self.value
  }

  pub fn write_timer_control(&mut self, value: u16, scheduler: &mut Scheduler) {
    let new_ctl = TimerControl::from_bits_retain(value);

    self.prescalar_frequency = CYCLE_LUT[new_ctl.prescalar_selection() as usize];

    let event_type = if self.is_arm9 {
      EventType::Timer9(self.id)
    } else {
      EventType::Timer7(self.id)
    };

    scheduler.remove(event_type);

    if new_ctl.contains(TimerControl::ENABLED) && !self.timer_ctl.contains(TimerControl::ENABLED) {
      self.value = self.reload_value;
      self.running = true;

      if !self.timer_ctl.contains(TimerControl::COUNT_UP_TIMING) {
        self.start_cycles = scheduler.cycles;

        let cycles_till_overflow = self.prescalar_frequency * (0x1_0000 - self.value as u32);

        scheduler.schedule(event_type, cycles_till_overflow as usize);
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