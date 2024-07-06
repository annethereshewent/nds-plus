use std::{rc::Rc, cell::Cell};

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

  // pub fn tick(&mut self, cycles: u32, dma: &mut DmaChannels) {
  //   for i in 0..self.t.len() {
  //     let timer = &mut self.t[i];

  //     let timer_overflowed = timer.tick(cycles);

  //     let timer_id = timer.id;
  //     if timer_overflowed {
  //       self.handle_overflow(timer_id, dma);
  //     }
  //   }
  // }

  pub fn handle_overflow(&mut self, timer_id: usize, dma: &mut DmaChannels, interrupt_request: &mut InterruptRequestRegister) {
    if timer_id != 3 {
      let next_timer_id = timer_id + 1;

      let next_timer = &mut self.t[next_timer_id];

      if next_timer.timer_ctl.contains(TimerControl::COUNT_UP_TIMING) && next_timer.count_up_timer(interrupt_request) {
        self.handle_overflow(next_timer_id, dma, interrupt_request);
      }
    }

    // add this back when you add the apu
    // if timer_id == 0 || timer_id == 1 {
    //   apu.handle_timer_overflow(timer_id, dma);
    // }
  }
}