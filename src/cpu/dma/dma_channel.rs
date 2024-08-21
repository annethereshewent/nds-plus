use registers::dma_control_register::DmaTiming;

use crate::scheduler::{EventType, Scheduler};

use self::registers::dma_control_register::DmaControlRegister;

pub mod registers;

#[derive(Copy, Clone, Debug)]
pub struct DmaParams {
  pub fifo_mode: bool,
  pub count: u32,
  pub word_size: i32,
  pub destination_adjust: i32,
  pub source_adjust: i32,
  pub should_trigger_irq: bool,
  pub source_address: u32,
  pub destination_address: u32
}

pub struct DmaChannel {
  id: usize,
  pub source_address: u32,
  pub destination_address: u32,
  pub internal_source_address: u32,
  pub internal_destination_address: u32,
  internal_count: u32,
  pub dma_control: DmaControlRegister,
  pub pending: bool,
  pub running: bool,
  fifo_mode: bool,
  is_arm9: bool
}

impl DmaChannel {
  pub fn new(id: usize, is_arm9: bool) -> Self {
    Self {
      source_address: 0,
      destination_address: 0,
      dma_control: DmaControlRegister::from_bits_retain(0),
      pending: false,
      internal_count: 0,
      internal_destination_address: 0,
      internal_source_address: 0,
      running: false,
      fifo_mode: false,
      id,
      is_arm9
    }
  }

  pub fn get_transfer_parameters(&mut self) -> DmaParams {
    let mut should_trigger_irq = false;

    let word_size = if self.dma_control.contains(DmaControlRegister::DMA_TRANSFER_TYPE) {
      4 // 32 bit
    } else {
      2 // 16 bit
    };

    let count = match self.internal_count {
      0 => if self.id == 3 { 0x1_0000 } else { 0x4000 },
      _ => self.internal_count as u32
    };

    let destination_adjust = match self.dma_control.dest_addr_control() {
      0 | 3 => word_size,
      1 => -word_size,
      2 => 0,
      _ => unreachable!("can't be")
    };

    let source_adjust = match self.dma_control.source_addr_control() {
      0 => word_size,
      1 => -word_size,
      2 => 0,
      _ => panic!("illegal value specified for source address control")
    };

    if self.dma_control.contains(DmaControlRegister::IRQ_ENABLE) {
      should_trigger_irq = true;
    }

    DmaParams {
      fifo_mode: self.fifo_mode,
      word_size,
      count,
      destination_adjust,
      source_adjust,
      destination_address: self.internal_destination_address,
      source_address: self.internal_source_address,
      should_trigger_irq
    }
  }

  pub fn write_source(&mut self, address: u32, mask: u32) {
    self.source_address &= mask;
    self.source_address |= address;
  }


  pub fn write_control(&mut self, value: u32, mask: Option<u32>, scheduler: &mut Scheduler) {
    let mut val = 0;

    if let Some(mask) = mask {
      val = self.dma_control.bits() & mask;
    }

    val |= value;

    let dma_control = DmaControlRegister::from_bits_retain(val);

    if dma_control.contains(DmaControlRegister::DMA_ENABLE) && !self.dma_control.contains(DmaControlRegister::DMA_ENABLE) {
      self.internal_destination_address = self.destination_address;
      self.internal_source_address = self.source_address;

      self.internal_count = dma_control.word_count();

      self.running = true;

      let timing = dma_control.dma_start_timing(self.is_arm9);

      if timing == DmaTiming::Immediately || timing == DmaTiming::GeometryCommandFifo {
        if timing == DmaTiming::Immediately {
          self.pending = true;
        } else {
          scheduler.schedule(EventType::CheckGeometryFifo, 1);
        }
      } else {
        self.pending = false;
      }
    }

    if !dma_control.contains(DmaControlRegister::DMA_ENABLE) {
      self.running = false;
    }

    self.dma_control = dma_control;
  }
}