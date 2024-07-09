use engine_2d::Engine2d;
use engine_3d::Engine3d;
use registers::{display_capture_control_register::DisplayCaptureControlRegister, display_status_register::{DispStatFlags, DisplayStatusRegister}, power_control_register1::PowerControlRegister1, power_control_register2::PowerControlRegister2, vram_control_register::VramControlRegister};
use vram::{Bank, VRam};

use crate::{cpu::{dma::{dma_channel::registers::dma_control_register::DmaTiming, dma_channels::DmaChannels}, registers::{interrupt_request_register::InterruptRequestRegister, mosaic_register::MosaicRegister}}, scheduler::{EventType, Scheduler}};

pub mod registers;
pub mod engine_2d;
pub mod engine_3d;
pub mod vram;

const CYCLES_PER_DOT: usize = 6;
const HBLANK_DOTS: usize = 256 + 8;
const DOTS_PER_LINE: usize = 355;

const NUM_LINES: u16 = 263;

pub const SCREEN_HEIGHT: u16 = 192;
pub const SCREEN_WIDTH: u16 = 256;

const BANK_A: u32 = Bank::BankA as u32;
const BANK_B: u32 = Bank::BankB as u32;
const BANK_C: u32 = Bank::BankC as u32;
const BANK_D: u32 = Bank::BankD as u32;
const BANK_E: u32 = Bank::BankE as u32;
const BANK_F: u32 = Bank::BankF as u32;
const BANK_G: u32 = Bank::BankG as u32;
const BANK_H: u32 = Bank::BankH as u32;
const BANK_I: u32 = Bank::BankI as u32;

#[derive(Copy, Clone)]
pub struct BgProps {
  pub x: i32,
  pub y: i32,
  pub dx: i16,
  pub dmx: i16,
  pub dy: i16,
  pub dmy: i16,
  pub internal_x: i32,
  pub internal_y: i32
}

impl BgProps {
  pub fn new() -> Self {
    Self {
      x: 0,
      y: 0,
      dx: 0,
      dmx: 0,
      dy: 0,
      dmy: 0,
      internal_x: 0,
      internal_y: 0
    }
  }
}

pub struct GPU {
  pub engine_a: Engine2d<false>,
  pub engine_b: Engine2d<true>,
  pub engine3d: Engine3d,
  pub powcnt1: PowerControlRegister1,
  pub powcnt2: PowerControlRegister2,
  pub vramcnt: [VramControlRegister; 9],
  pub dispstat: [DisplayStatusRegister; 2],
  pub frame_finished: bool,
  pub vram: VRam,
  pub vcount: u16,
  pub dispcapcnt: DisplayCaptureControlRegister,
  pub mosaic: MosaicRegister
}

impl GPU {
  pub fn new(scheduler: &mut Scheduler) -> Self {
    let mut vramcnt: Vec<VramControlRegister> = Vec::new();

    for i in 0..9 {
      vramcnt.push(VramControlRegister::new(i));
    }

    let gpu = Self {
      engine_a: Engine2d::new(),
      engine_b: Engine2d::new(),
      engine3d: Engine3d::new(),
      powcnt1: PowerControlRegister1::from_bits_retain(0),
      powcnt2: PowerControlRegister2::from_bits_retain(0),
      vramcnt: vramcnt.try_into().unwrap(),
      dispstat: [DisplayStatusRegister::new(), DisplayStatusRegister::new()],
      dispcapcnt: DisplayCaptureControlRegister::new(),
      vcount: 0,
      frame_finished: false,
      vram: VRam::new(),
      mosaic: MosaicRegister::new()
    };

    scheduler.schedule(EventType::HBlank, CYCLES_PER_DOT * HBLANK_DOTS);

    gpu
  }

  pub fn handle_hblank(&mut self, scheduler: &mut Scheduler, interrupt_requests: &mut [&mut InterruptRequestRegister], dma_channels: &mut [&mut DmaChannels]) {
    self.schedule_next_line(scheduler);

    for dispstat in &mut self.dispstat {
      dispstat.flags.insert(DispStatFlags::HBLANK);
    }

    for dma in dma_channels {
      dma.notify_gpu_event(DmaTiming::Hblank);
    }

    if self.vcount < SCREEN_HEIGHT {
      self.render_line();
    }

    self.check_interrupts(DispStatFlags::HBLANK_IRQ_ENABLE, InterruptRequestRegister::HBLANK, interrupt_requests);
  }

  pub fn check_interrupts(&mut self, dispstat_flag: DispStatFlags, interrupt_flag: InterruptRequestRegister, interrupt_requests: &mut [&mut InterruptRequestRegister]) {
    for i in 0..2 {
      let dispstat = &mut self.dispstat[i];
      let interrupt_request = &mut interrupt_requests[i];

      if dispstat.flags.contains(dispstat_flag) {
        interrupt_request.insert(interrupt_flag);
      }
    }
  }

  pub fn start_next_line(&mut self, scheduler: &mut Scheduler, interrupt_requests: &mut [&mut InterruptRequestRegister], dma_channels: &mut [&mut DmaChannels]) {
    scheduler.schedule(EventType::HBlank, CYCLES_PER_DOT * HBLANK_DOTS);

    self.vcount += 1;

    if self.vcount == NUM_LINES {
      self.vcount = 0;
    }

    if self.vcount == 0 {
      // TODO: dispcapcnt register stuff

      for dispstat in &mut self.dispstat {
        dispstat.flags.remove(DispStatFlags::VBLANK);
      }
    } else if self.vcount == SCREEN_HEIGHT {
      self.trigger_vblank();

      for dma in dma_channels {
        dma.notify_gpu_event(DmaTiming::Vblank);
      }

      self.frame_finished = true;
      self.check_interrupts(DispStatFlags::VBLANK_IRQ_ENABLE, InterruptRequestRegister::VBLANK, interrupt_requests);
    }

    for i in 0..2 {
      let dispstat = &mut self.dispstat[i];
      let interrupt_request = &mut interrupt_requests[i];

      if dispstat.flags.contains(DispStatFlags::VCOUNTER_IRQ_ENABLE) && self.vcount == dispstat.vcount_setting {
        interrupt_request.insert(InterruptRequestRegister::VCOUNTER_MATCH);
      }
    }

  }

  pub fn write_palette_a(&mut self, address: u32, val: u8) {
    self.engine_a.write_palette_ram(address, val);
  }

  pub fn write_palette_b(&mut self, address: u32, val: u8) {
    self.engine_b.write_palette_ram(address, val);
  }

  pub fn write_lcdc(&mut self, address: u32, val: u8) {
    match address {
      0x680_0000..=0x681_ffff => self.vram.write_lcdc_bank(Bank::BankA, address, val),
      0x682_0000..=0x683_ffff => self.vram.write_lcdc_bank(Bank::BankB, address, val),
      0x684_0000..=0x685_ffff => self.vram.write_lcdc_bank(Bank::BankC, address, val),
      0x686_0000..=0x687_ffff => self.vram.write_lcdc_bank(Bank::BankD, address, val),
      0x688_0000..=0x688_ffff => self.vram.write_lcdc_bank(Bank::BankE, address, val),
      0x689_0000..=0x689_3fff => self.vram.write_lcdc_bank(Bank::BankF, address, val),
      0x689_4000..=0x689_7fff => self.vram.write_lcdc_bank(Bank::BankG, address, val),
      0x689_8000..=0x689_ffff => self.vram.write_lcdc_bank(Bank::BankH, address, val),
      0x68a_0000..=0x68a_3fff => self.vram.write_lcdc_bank(Bank::BankI, address, val),
      _ => unreachable!("received address: {:X}", address)
    }
  }

  pub fn read_lcdc(&mut self, address: u32) -> u8 {
    match address {
      0x680_0000..=0x681_ffff => self.vram.read_lcdc_bank(Bank::BankA, address),
      0x682_0000..=0x683_ffff => self.vram.read_lcdc_bank(Bank::BankB, address),
      0x684_0000..=0x685_ffff => self.vram.read_lcdc_bank(Bank::BankC, address),
      0x686_0000..=0x687_ffff => self.vram.read_lcdc_bank(Bank::BankD, address),
      0x688_0000..=0x688_ffff => self.vram.read_lcdc_bank(Bank::BankE, address),
      0x689_0000..=0x689_3fff => self.vram.read_lcdc_bank(Bank::BankF, address),
      0x689_4000..=0x689_7fff => self.vram.read_lcdc_bank(Bank::BankG, address),
      0x689_8000..=0x689_ffff => self.vram.read_lcdc_bank(Bank::BankH, address),
      0x68a_0000..=0x68a_3fff => self.vram.read_lcdc_bank(Bank::BankI, address),
      _ => unreachable!("received address: {:X}", address)
    }
  }

  pub fn read_arm7_wram(&self, address: u32) -> u8 {
    self.vram.read_arm7_wram(address)
  }

  pub fn write_vramcnt(&mut self, offset: u32, val: u8) {
    if self.vramcnt[offset as usize].vram_enable {
      match offset {
        BANK_A => self.vram.unmap_bank(Bank::BankA, &self.vramcnt[offset as usize]),
        BANK_B => self.vram.unmap_bank(Bank::BankB, &self.vramcnt[offset as usize]),
        BANK_C => self.vram.unmap_bank(Bank::BankC, &self.vramcnt[offset as usize]),
        BANK_D => self.vram.unmap_bank(Bank::BankD, &self.vramcnt[offset as usize]),
        BANK_E => self.vram.unmap_bank(Bank::BankE, &self.vramcnt[offset as usize]),
        BANK_F => self.vram.unmap_bank(Bank::BankF, &self.vramcnt[offset as usize]),
        BANK_G => self.vram.unmap_bank(Bank::BankG, &self.vramcnt[offset as usize]),
        BANK_H => self.vram.unmap_bank(Bank::BankH, &self.vramcnt[offset as usize]),
        BANK_I => self.vram.unmap_bank(Bank::BankI, &self.vramcnt[offset as usize]),
        _ => unreachable!("can't happen")
      }
    }

    self.vramcnt[offset as usize].write(val);

    if self.vramcnt[offset as usize].vram_enable {
      match offset {
        BANK_A => self.vram.map_bank(Bank::BankA, &self.vramcnt[offset as usize]),
        BANK_B => self.vram.map_bank(Bank::BankB, &self.vramcnt[offset as usize]),
        BANK_C => self.vram.map_bank(Bank::BankC, &self.vramcnt[offset as usize]),
        BANK_D => self.vram.map_bank(Bank::BankD, &self.vramcnt[offset as usize]),
        BANK_E => self.vram.map_bank(Bank::BankE, &self.vramcnt[offset as usize]),
        BANK_F => self.vram.map_bank(Bank::BankF, &self.vramcnt[offset as usize]),
        BANK_G => self.vram.map_bank(Bank::BankG, &self.vramcnt[offset as usize]),
        BANK_H => self.vram.map_bank(Bank::BankH, &self.vramcnt[offset as usize]),
        BANK_I => self.vram.map_bank(Bank::BankI, &self.vramcnt[offset as usize]),
        _ => todo!("unimplemented")
      }
    }
  }

  pub fn read_vramcnt(&self, offset: u32) -> u8 {
    self.vramcnt[offset as usize].read()
  }

  pub fn get_arm7_vram_stat(&self) -> u8 {
    ((self.vramcnt[2].vram_enable && self.vramcnt[2].vram_mst == 2) as u8) | ((self.vramcnt[3].vram_enable && self.vramcnt[3].vram_mst == 2) as u8) << 1
  }

  pub fn schedule_next_line(&mut self, scheduler: &mut Scheduler) {
    scheduler.schedule(EventType::NextLine, CYCLES_PER_DOT * DOTS_PER_LINE);
  }

  fn trigger_vblank(&mut self) {
    for dispstat in &mut self.dispstat {
      dispstat.flags.insert(DispStatFlags::VBLANK);
    }
    // TODO: 3d rendering and possibly other stuff
  }

  fn render_line(&mut self) {
    if self.powcnt1.contains(PowerControlRegister1::ENGINE_A_ENABLE) {
      self.engine_a.render_line(self.vcount, &mut self.vram);
    }
    if self.powcnt1.contains(PowerControlRegister1::ENGINE_B_ENABLE) {
      self.engine_b.render_line(self.vcount, &mut self.vram);
    }
  }
}