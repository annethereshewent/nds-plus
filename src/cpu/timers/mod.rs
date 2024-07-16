use crate::scheduler::Scheduler;

use self::timer::{Timer, TimerControl};

use super::{dma::dma_channels::DmaChannels, registers::interrupt_request_register::InterruptRequestRegister};

pub mod timer;

pub struct Timers {
  pub t: [Timer; 4],
}

impl Timers {
  pub fn new(is_arm9: bool) -> Self {
    Self {
      t: [Timer::new(0, is_arm9), Timer::new(1, is_arm9), Timer::new(2, is_arm9), Timer::new(3, is_arm9)],
    }
  }

  pub fn handle_overflow(&mut self, timer_id: usize, dma: &mut DmaChannels, interrupt_request: &mut InterruptRequestRegister, scheduler: &mut Scheduler, cycles_left: usize) {
    if timer_id != 3 {
      let next_timer_id = timer_id + 1;

      let next_timer = &mut self.t[next_timer_id];

      if next_timer.timer_ctl.contains(TimerControl::COUNT_UP_TIMING) && next_timer.count_up_timer(interrupt_request, scheduler, cycles_left) {
        self.handle_overflow(next_timer_id, dma, interrupt_request, scheduler, cycles_left);
      }
    }
  }
}