use std::{cell::RefCell, cmp::Reverse, rc::Rc};

use crate::{cpu::{bus::Bus, CPU}, scheduler::{EventType, Scheduler}};

pub struct Nds {
  pub arm9_cpu: CPU<true>,
  pub arm7_cpu: CPU<false>,
  scheduler: Rc<RefCell<Scheduler>>,
  pub bus: Rc<RefCell<Bus>>
}

impl Nds {
  pub fn new(firmware_bytes: Vec<u8>, bios7_bytes: Vec<u8>, bios9_bytes: Vec<u8>, rom_bytes: Vec<u8>, skip_bios: bool) -> Self {
    let scheduler = Rc::new(RefCell::new(Scheduler::new()));
    let bus = Rc::new(
      RefCell::new(
        Bus::new(
          firmware_bytes,
          bios7_bytes,
          bios9_bytes,
          rom_bytes,
          skip_bios,
          scheduler.clone()
        )
      )
    );
    let mut nds = Self {
      arm9_cpu: CPU::new(bus.clone(), skip_bios),
      arm7_cpu: CPU::new(bus.clone(), skip_bios),
      scheduler,
      bus
    };

    nds.arm7_cpu.reload_pipeline32();
    nds.arm9_cpu.reload_pipeline32();

    nds
  }

  pub fn step(&mut self) -> bool {
    let ref mut scheduler = *self.scheduler.borrow_mut();

    let mut frame_finished = false;

    if let Some((event_type, cycles)) = scheduler.get_next_event() {
      self.arm9_cpu.step(cycles * 2);
      self.arm7_cpu.step(cycles);

      // finally handle any events
      let ref mut bus = *self.bus.borrow_mut();

      match event_type {
        EventType::HBLANK => bus.gpu.handle_hblank(scheduler),
        EventType::NEXT_LINE => bus.gpu.start_next_line(scheduler)
      }

      scheduler.update_cycles(cycles);

      frame_finished = bus.gpu.frame_finished;
    } else {
      panic!("there are no events left to process! something probably went wrong");
    }

    frame_finished
  }

  pub fn start_new_frame(&mut self) {
    let ref mut bus = *self.bus.borrow_mut();

    bus.gpu.frame_finished = false;
  }
}