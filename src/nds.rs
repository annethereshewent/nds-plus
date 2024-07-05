use std::{cell::RefCell, rc::Rc};

use crate::{cpu::{bus::Bus, CPU}, scheduler::{EventType, Scheduler}};

pub struct Nds {
  pub arm9_cpu: CPU<true>,
  pub arm7_cpu: CPU<false>,
  scheduler: Scheduler,
  pub bus: Rc<RefCell<Bus>>
}

impl Nds {
  pub fn new(firmware_bytes: Vec<u8>, bios7_bytes: Vec<u8>, bios9_bytes: Vec<u8>, rom_bytes: Vec<u8>, skip_bios: bool) -> Self {
    let mut scheduler = Scheduler::new();
    let bus = Rc::new(
      RefCell::new(
        Bus::new(
          firmware_bytes,
          bios7_bytes,
          bios9_bytes,
          rom_bytes,
          skip_bios,
          &mut scheduler
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
    let mut frame_finished = false;

    if let Some((event_type, cycles)) = self.scheduler.get_next_event() {
      self.arm9_cpu.step(cycles * 2);
      self.arm7_cpu.step(cycles);

      // finally handle any events
      let ref mut bus = *self.bus.borrow_mut();

      let mut interrupt_requests = [&mut bus.arm7.interrupt_request, &mut bus.arm9.interrupt_request];
      let mut dma_channels = [&mut bus.arm7.dma_channels, &mut bus.arm9.dma_channels];

      match event_type {
        EventType::HBLANK => bus.gpu.handle_hblank(&mut self.scheduler, &mut interrupt_requests, &mut dma_channels),
        EventType::NEXT_LINE => bus.gpu.start_next_line(&mut self.scheduler, &mut interrupt_requests, &mut dma_channels),
        _ => todo!("not implemented yet")
      }

      self.scheduler.update_cycles(cycles);

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