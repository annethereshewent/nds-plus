use std::{cell::RefCell, rc::Rc};

use crate::{cpu::{bus::Bus, CPU}, scheduler::EventType};

pub struct Nds {
  pub arm9_cpu: CPU<true>,
  pub arm7_cpu: CPU<false>,
  pub bus: Rc<RefCell<Bus>>
}

impl Nds {
  pub fn new(firmware_bytes: Vec<u8>, bios7_bytes: Vec<u8>, bios9_bytes: Vec<u8>, rom_bytes: Vec<u8>, skip_bios: bool) -> Self {
    let bus = Rc::new(
      RefCell::new(
        Bus::new(
          firmware_bytes,
          bios7_bytes,
          bios9_bytes,
          rom_bytes,
          skip_bios
        )
      )
    );
    let mut nds = Self {
      arm9_cpu: CPU::new(bus.clone(), skip_bios),
      arm7_cpu: CPU::new(bus.clone(), skip_bios),
      bus
    };

    nds.arm7_cpu.reload_pipeline32();
    nds.arm9_cpu.reload_pipeline32();

    nds
  }

  pub fn step(&mut self) -> bool {


    let mut cycles = 0;
    let mut scheduler_cycles = 0;

    // Rust forcing me to do weird shit haha
    {
      let ref mut bus = *self.bus.borrow_mut();
      cycles = bus.scheduler.get_cycles_to_next_event();
      scheduler_cycles = bus.scheduler.cycles;
    }

    let actual_target = std::cmp::min(scheduler_cycles + 30, cycles);

    self.arm9_cpu.step(actual_target * 2);
    self.arm7_cpu.step(actual_target);

    let ref mut bus = *self.bus.borrow_mut();

    bus.scheduler.update_cycles(actual_target);

    // finally check if there are any events to handle.
    while let Some(event_type) = bus.scheduler.get_next_event() {
      let mut interrupt_requests = [&mut bus.arm7.interrupt_request, &mut bus.arm9.interrupt_request];
      let mut dma_channels = [&mut bus.arm7.dma, &mut bus.arm9.dma];

      match event_type {
        EventType::HBlank => bus.gpu.handle_hblank(&mut bus.scheduler, &mut interrupt_requests, &mut dma_channels),
        EventType::NextLine => bus.gpu.start_next_line(&mut bus.scheduler, &mut interrupt_requests, &mut dma_channels),
        EventType::DMA7(channel_id) => bus.arm7.dma.channels[channel_id].pending = true,
        EventType::DMA9(channel_id) => bus.arm9.dma.channels[channel_id].pending = true,
        EventType::Timer7(timer_id) => bus.arm7.timers.handle_overflow(timer_id, &mut bus.arm7.dma, &mut bus.arm7.interrupt_request),
        EventType::Timer9(timer_id) => bus.arm9.timers.handle_overflow(timer_id, &mut bus.arm9.dma, &mut bus.arm9.interrupt_request),
        EventType::BlockFinished(is_arm9) if is_arm9 => bus.cartridge.on_block_finished(&mut bus.arm9.interrupt_request),
        EventType::WordTransfer(is_arm9) if is_arm9 => bus.cartridge.on_word_transferred(&mut bus.arm9.dma),
        EventType::WordTransfer(_) => bus.cartridge.on_word_transferred(&mut bus.arm7.dma),
        EventType::BlockFinished(_) => bus.cartridge.on_block_finished(&mut bus.arm7.interrupt_request)
      }
    }

    bus.gpu.frame_finished
  }
}