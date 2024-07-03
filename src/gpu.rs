use std::{cell::RefCell, rc::Rc};

use engine_2d::Engine2d;
use engine_3d::Engine3d;
use registers::{display_status_register::{DispStatFlags, DisplayStatusRegister}, power_control_register1::PowerControlRegister1, power_control_register2::PowerControlRegister2, vram_control_register::VramControlRegister};
use vram::{Bank, VRam};

use crate::scheduler::{EventType, Scheduler};

pub mod registers;
pub mod engine_2d;
pub mod engine_3d;
pub mod vram;

const CYCLES_PER_DOT: usize = 6;
const HBLANK_DOTS: usize = 256 + 8;
const DOTS_PER_LINE: usize = 355;

const NUM_LINES: u16 = 263;

pub const HEIGHT: u16 = 192;
pub const WIDTH: u16 = 256;

const BANK_A: u32 = Bank::BankA as u32;
const BANK_B: u32 = Bank::BankB as u32;
const BANK_C: u32 = Bank::BankC as u32;
const BANK_D: u32 = Bank::BankD as u32;
const BANK_E: u32 = Bank::BankE as u32;
const BANK_F: u32 = Bank::BankF as u32;
const BANK_G: u32 = Bank::BankG as u32;
const BANK_H: u32 = Bank::BankH as u32;
const BANK_I: u32 = Bank::BankI as u32;

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
  scheduler: Rc<RefCell<Scheduler>>,
  vcount: u16
}

impl GPU {
  pub fn new(scheduler: Rc<RefCell<Scheduler>>) -> Self {
    let mut vramcnt: Vec<VramControlRegister> = Vec::new();

    for i in 0..9 {
      vramcnt.push(VramControlRegister::new(i));
    }

    let mut gpu = Self {
      engine_a: Engine2d::new(),
      engine_b: Engine2d::new(),
      engine3d: Engine3d::new(),
      powcnt1: PowerControlRegister1::from_bits_retain(0),
      powcnt2: PowerControlRegister2::from_bits_retain(0),
      vramcnt: vramcnt.try_into().unwrap(),
      dispstat: [DisplayStatusRegister::new(), DisplayStatusRegister::new()],
      scheduler,
      vcount: 0,
      frame_finished: false,
      vram: VRam::new()
    };

    gpu.schedule_hblank();

    gpu
  }

  fn schedule_hblank(&mut self) {
    let ref mut scheduler = *self.scheduler.borrow_mut();

    scheduler.schedule(EventType::HBLANK, CYCLES_PER_DOT * HBLANK_DOTS);
  }

  pub fn handle_hblank(&mut self, scheduler: &mut Scheduler) {
    self.schedule_next_line(scheduler);

    for dispstat in &mut self.dispstat {
      dispstat.flags.insert(DispStatFlags::HBLANK);
    }

    if self.vcount < HEIGHT {
      self.render_line();
    }

    // TODO: check for hblank interrupt
  }

  pub fn start_next_line(&mut self, scheduler: &mut Scheduler) {
    scheduler.schedule(EventType::HBLANK, CYCLES_PER_DOT * HBLANK_DOTS);

    self.vcount += 1;

    if self.vcount == NUM_LINES {
      self.vcount = 0;
    }

    if self.vcount == 0 {
      // TODO: dispcapcnt register stuff
      for dispstat in &mut self.dispstat {
        dispstat.flags.remove(DispStatFlags::VBLANK);


      }
    } else if self.vcount == HEIGHT {
      self.trigger_vblank();

      self.frame_finished = true;
      // TODO: check for vblank interrupt here
    }

    // TODO: check for vcounter interrupt here
  }

  pub fn write_lcdc(&mut self, address: u32, val: u8) {
    match address {
      0x680_0000..=0x681_ffff => self.vram.write_lcdc_bank(Bank::BankA, address, val, 0x1_ffff),
      0x682_0000..=0x683_ffff => self.vram.write_lcdc_bank(Bank::BankB, address, val, 0x1_ffff),
      0x684_0000..=0x685_ffff => self.vram.write_lcdc_bank(Bank::BankC, address, val, 0x1_ffff),
      0x686_0000..=0x687_ffff => self.vram.write_lcdc_bank(Bank::BankD, address, val, 0x1_ffff),
      0x688_0000..=0x688_ffff => self.vram.write_lcdc_bank(Bank::BankE, address, val, 0xffff),
      0x689_0000..=0x689_3fff => self.vram.write_lcdc_bank(Bank::BankF, address, val, 0x3fff),
      0x689_4000..=0x689_7fff => self.vram.write_lcdc_bank(Bank::BankG, address, val, 0x3fff),
      0x689_8000..=0x689_ffff => self.vram.write_lcdc_bank(Bank::BankH, address, val, 0x7fff),
      0x68a_0000..=0x68a_3fff => self.vram.write_lcdc_bank(Bank::BankI, address, val, 0x3fff),
      _ => unreachable!("received address: {:X}", address)
    }
  }

  pub fn write_vramcnt(&mut self, offset: u32, val: u8) {
    if self.vramcnt[offset as usize].vram_enable {
      match offset {
        BANK_A => self.vram.unmap_bank(Bank::BankA, self.vramcnt[offset as usize].vram_mst),
        BANK_B => self.vram.unmap_bank(Bank::BankB, self.vramcnt[offset as usize].vram_mst),
        BANK_C => self.vram.unmap_bank(Bank::BankC, self.vramcnt[offset as usize].vram_mst),
        BANK_D => self.vram.unmap_bank(Bank::BankD, self.vramcnt[offset as usize].vram_mst),
        BANK_E => self.vram.unmap_bank(Bank::BankE, self.vramcnt[offset as usize].vram_mst),
        BANK_F => self.vram.unmap_bank(Bank::BankF, self.vramcnt[offset as usize].vram_mst),
        BANK_G => self.vram.unmap_bank(Bank::BankG, self.vramcnt[offset as usize].vram_mst),
        BANK_H => self.vram.unmap_bank(Bank::BankH, self.vramcnt[offset as usize].vram_mst),
        BANK_I => self.vram.unmap_bank(Bank::BankI, self.vramcnt[offset as usize].vram_mst),
        _ => unreachable!("can't happen")
      }
    }

    self.vramcnt[offset as usize].write(val);

    if self.vramcnt[offset as usize].vram_enable {
      match offset {
        BANK_A => self.vram.map_bank(Bank::BankA, self.vramcnt[offset as usize].vram_mst),
        BANK_B => self.vram.map_bank(Bank::BankB, self.vramcnt[offset as usize].vram_mst),
        BANK_C => self.vram.map_bank(Bank::BankC, self.vramcnt[offset as usize].vram_mst),
        BANK_D => self.vram.map_bank(Bank::BankD, self.vramcnt[offset as usize].vram_mst),
        BANK_E => self.vram.map_bank(Bank::BankE, self.vramcnt[offset as usize].vram_mst),
        BANK_F => self.vram.map_bank(Bank::BankF, self.vramcnt[offset as usize].vram_mst),
        BANK_G => self.vram.map_bank(Bank::BankG, self.vramcnt[offset as usize].vram_mst),
        BANK_H => self.vram.map_bank(Bank::BankH, self.vramcnt[offset as usize].vram_mst),
        BANK_I => self.vram.map_bank(Bank::BankI, self.vramcnt[offset as usize].vram_mst),
        _ => todo!("unimplemented")
      }
    }
  }

  pub fn schedule_next_line(&mut self, scheduler: &mut Scheduler) {
    scheduler.schedule(EventType::NEXT_LINE, CYCLES_PER_DOT * DOTS_PER_LINE);
  }

  fn trigger_vblank(&mut self) {
    for dispstat in &mut self.dispstat {
      dispstat.flags.insert(DispStatFlags::VBLANK);
    }

    // do some other stuff here
  }

  fn render_line(&mut self) {
    if self.powcnt1.contains(PowerControlRegister1::ENGINE_A_ENABLE) {
      self.engine_a.render_line(self.vcount,&mut self.vram);
    }
  }
}