use std::{cell::RefCell, collections::VecDeque, fs::File, path::PathBuf, rc::Rc, sync::{Arc, Mutex}};

use crate::{cpu::{bus::Bus, CPU}, scheduler::EventType};

pub struct Nds {
  pub arm9_cpu: CPU<true>,
  pub arm7_cpu: CPU<false>,
  pub bus: Rc<RefCell<Bus>>
}

impl Nds {
  pub fn new(
    file_path: String,
    firmware_path: PathBuf,
    bios7_bytes: Vec<u8>,
    bios9_bytes: Vec<u8>,
    rom_bytes: Vec<u8>,
    skip_bios: bool,
    audio_buffer: Arc<Mutex<VecDeque<f32>>>) -> Self
  {
    let bus = Rc::new(
      RefCell::new(
        Bus::new(
          file_path,
          firmware_path,
          bios7_bytes,
          bios9_bytes,
          rom_bytes,
          skip_bios,
          audio_buffer
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
    // Rust forcing me to do weird shit haha
    let (cycles, scheduler_cycles) = {
      let ref mut bus = *self.bus.borrow_mut();
      let cycles = bus.scheduler.get_cycles_to_next_event();
      let scheduler_cycles = bus.scheduler.cycles;
      (cycles, scheduler_cycles)
    };

    let actual_target = std::cmp::min(scheduler_cycles + 30, cycles);

    self.arm9_cpu.step(actual_target * 2);
    self.arm7_cpu.step(actual_target);

    let ref mut bus = *self.bus.borrow_mut();

    bus.scheduler.update_cycles(actual_target);

    // finally check if there are any events to handle.
    while let Some((event_type, cycles_left)) = bus.scheduler.get_next_event() {
      let mut interrupt_requests = [&mut bus.arm7.interrupt_request, &mut bus.arm9.interrupt_request];
      let mut dma_channels = [&mut bus.arm7.dma, &mut bus.arm9.dma];

      match event_type {
        EventType::HBlank => bus.gpu.handle_hblank(&mut bus.scheduler, &mut interrupt_requests, &mut dma_channels, cycles_left),
        EventType::HDraw => bus.gpu.start_next_line(&mut bus.scheduler, &mut interrupt_requests, &mut dma_channels, cycles_left),
        EventType::DMA7(channel_id) => bus.arm7.dma.channels[channel_id].pending = true,
        EventType::DMA9(channel_id) => bus.arm9.dma.channels[channel_id].pending = true,
        EventType::Timer7(timer_id) => {
          let timers = &mut bus.arm7.timers;

          timers.t[timer_id].handle_overflow(&mut bus.arm7.interrupt_request, &mut bus.scheduler, cycles_left);
          timers.handle_overflow(timer_id, &mut bus.arm7.dma, &mut bus.arm7.interrupt_request, &mut bus.scheduler, cycles_left);
        }
        EventType::Timer9(timer_id) => {
          let timers = &mut bus.arm9.timers;

          timers.t[timer_id].handle_overflow(&mut bus.arm9.interrupt_request, &mut bus.scheduler, cycles_left);
          timers.handle_overflow(timer_id, &mut bus.arm9.dma, &mut bus.arm9.interrupt_request, &mut bus.scheduler, cycles_left);
        }
        EventType::BlockFinished(is_arm9) if is_arm9 => bus.cartridge.on_block_finished(&mut bus.arm9.interrupt_request),
        EventType::WordTransfer(is_arm9) if is_arm9 => bus.cartridge.on_word_transferred(&mut bus.arm9.dma),
        EventType::WordTransfer(_) => bus.cartridge.on_word_transferred(&mut bus.arm7.dma),
        EventType::BlockFinished(_) => bus.cartridge.on_block_finished(&mut bus.arm7.interrupt_request),
        EventType::StepAudio(channel_id) => bus.step_audio(channel_id, cycles_left),
        EventType::ResetAudio(channel_id) => bus.arm7.apu.channels[channel_id].reset_audio(),
        EventType::GenerateSample => bus.arm7.apu.generate_samples(&mut bus.scheduler, cycles_left)
      }
    }

    bus.gpu.frame_finished
  }
}