use crate::scheduler::Scheduler;

use super::dma_channel::{registers::dma_control_register::{DmaControlRegister, DmaTiming}, DmaChannel, DmaParams};

// pub const VBLANK_TIMING: u16 = 1;
// pub const HBLANK_TIMING: u16 = 2;
// const FIFO_TIMING: u16 = 3;

#[derive(Copy, Clone)]
pub struct DmaChannels {
  pub channels: [DmaChannel; 4],
  is_arm9: bool
}

pub enum AddressType {
  Low,
  High
}

impl DmaChannels{
  pub fn new(is_arm9: bool) -> Self {
    Self {
      channels: [
        DmaChannel::new(0, is_arm9),
        DmaChannel::new(1, is_arm9),
        DmaChannel::new(2, is_arm9),
        DmaChannel::new(3, is_arm9)
      ],
      is_arm9
    }
  }

  pub fn notify_gpu_event(&mut self, timing: DmaTiming) {
    for channel in &mut self.channels {
      if channel.dma_control.contains(DmaControlRegister::DMA_ENABLE) && channel.dma_control.dma_start_timing(self.is_arm9) == timing {
        channel.pending = true;
      }
    }
  }

  pub fn notify_cartridge_event(&mut self) {
    for channel in &mut self.channels {
      if channel.dma_control.contains(DmaControlRegister::DMA_ENABLE) && channel.dma_control.dma_start_timing(self.is_arm9) == DmaTiming::DSCartridgeSlot {
        channel.pending = true;
      }
    }
  }

  pub fn write(&mut self, channel: usize, index: usize, val: u32, mask: Option<u32>, scheduler: &mut Scheduler) {

    match index {
      0x0 => {
        let mut value = 0;

        if let Some(mask) = mask {
          value = self.channels[channel].source_address & mask;
        }

        value |= val;

        self.channels[channel].source_address = value;
      }
      0x4 => {
        let mut value = 0;

        if let Some(mask) = mask {
          value = self.channels[channel].destination_address & mask;
        }

        value |= val;

        self.channels[channel].destination_address = value;
      }
      0x8 => self.channels[channel].write_control(val, mask, scheduler),
      _ => panic!("invalid index given for dma write method")
    }
  }

  pub fn read(&mut self, channel: usize, index: usize) -> u32 {
    match index {
      0x0 => self.channels[channel].source_address,
      0x4 => self.channels[channel].destination_address,
      0x8 => self.channels[channel].dma_control.bits(),
      _ => panic!("invalid index given for dma write method")
    }
  }

  pub fn notify_apu_event(&mut self, address: u32) {
    for channel in &mut self.channels {
      if channel.dma_control.contains(DmaControlRegister::DMA_ENABLE)
        && channel.running
        && channel.dma_control.dma_start_timing(self.is_arm9) == DmaTiming::Fifo
        && channel.destination_address == address {
          channel.pending = true;
        }
    }
  }

  pub fn notify_geometry_fifo_event(&mut self) {
    for channel in &mut self.channels {
      if channel.dma_control.contains(DmaControlRegister::DMA_ENABLE) && channel.dma_control.dma_start_timing(self.is_arm9) == DmaTiming::GeometryCommandFifo {
        channel.pending = true;
      }
    }
  }

  pub fn get_transfer_parameters(&mut self) -> Vec<Option<DmaParams>> {
    let mut dma_params = Vec::new();

    for channel in &mut self.channels {
      if channel.pending {
        let params = channel.get_transfer_parameters();

        dma_params.push(Some(params));
        channel.pending = false;
      } else {
        dma_params.push(None);
      }
    }

    dma_params
  }

  pub fn has_pending_transfers(&self) -> bool {
    for channel in &self.channels {
      if channel.pending {
        return true;
      }
    }

    false
  }

  pub fn set_source_address(&mut self, channel_id: usize, value: u16, address_type: AddressType) {
    match address_type {
      AddressType::Low => {
        self.channels[channel_id].source_address = (self.channels[channel_id].source_address & 0xffff0000) | (value as u32);
      }
      AddressType::High => {
        self.channels[channel_id].source_address = (self.channels[channel_id].source_address & 0xffff) | ((value & 0xfff) as u32) << 16
      }
    }
  }

  pub fn set_destination_address(&mut self, channel_id: usize, value: u16, address_type: AddressType) {
    match address_type {
      AddressType::Low => {
        self.channels[channel_id].destination_address = (self.channels[channel_id].destination_address & 0xffff0000) | (value as u32);
      }
      AddressType::High => {
        self.channels[channel_id].destination_address = (self.channels[channel_id].destination_address & 0xffff) | ((value & 0xfff )as u32) << 16
      }
    }
  }
}