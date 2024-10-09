use std::{
  cell::RefCell, collections::VecDeque, fs, path::PathBuf, rc::Rc, sync::{
    Arc,
    Mutex
  }
};

use crate::{
  cpu::{
    bus::{cartridge::Header, Bus},
    CPU
  },
  scheduler::EventType
};

pub struct Nds {
  pub arm9_cpu: CPU<true>,
  pub arm7_cpu: CPU<false>,
  pub bus: Rc<RefCell<Bus>>,
  pub mic_samples: Arc<Mutex<[i16; 2048]>>
}

impl Nds {
  pub fn new(
    firmware_path: Option<PathBuf>,
    firmware_bytes: Option<Vec<u8>>,
    bios7_bytes: Vec<u8>,
    bios9_bytes: Vec<u8>,
    audio_buffer: Arc<Mutex<VecDeque<f32>>>,
    mic_samples: Arc<Mutex<[i16; 2048]>>
  ) -> Self {
    let bus = Rc::new(
      RefCell::new(
        Bus::new(
          firmware_path,
          firmware_bytes,
          bios7_bytes,
          bios9_bytes,
          audio_buffer
        )
      )
    );
    let mut nds = Self {
      arm9_cpu: CPU::new(bus.clone()),
      arm7_cpu: CPU::new(bus.clone()),
      bus,
      mic_samples
    };

    nds.arm7_cpu.reload_pipeline32();
    nds.arm9_cpu.reload_pipeline32();

    nds
  }

  pub fn reset(&mut self, rom: &Vec<u8>) {
    {
      let ref mut bus = *self.bus.borrow_mut();

      bus.arm7.apu.audio_buffer.lock().unwrap().drain(..);

      let mut new_bus = bus.reset();

      new_bus.cartridge.rom = rom.clone();
      new_bus.cartridge.header = Header::from(rom);

      new_bus.skip_bios();

      let bus_rc = Rc::new(RefCell::new(new_bus));

      self.arm9_cpu = CPU::new(bus_rc.clone());
      self.arm7_cpu = CPU::new(bus_rc.clone());

      self.arm9_cpu.skip_bios();
      self.arm7_cpu.skip_bios();

      self.arm7_cpu.reload_pipeline32();
      self.arm9_cpu.reload_pipeline32();
    }

    self.bus = self.arm9_cpu.bus.clone();
  }

  pub fn step(&mut self) -> bool {
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
        EventType::GenerateSample => bus.arm7.apu.generate_samples(&mut bus.scheduler, cycles_left),
        EventType::CheckGeometryFifo => {
          if bus.gpu.engine3d.should_run_dmas() {
            bus.arm9.dma.notify_geometry_fifo_event();
            bus.arm7.dma.notify_geometry_fifo_event();
          }
        }
      }
    }

    bus.gpu.frame_finished
  }
}