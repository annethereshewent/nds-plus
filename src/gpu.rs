use std::{cell::RefCell, rc::Rc};

use engine_2d::Engine2d;
use engine_3d::Engine3d;
use registers::{display_status_register::{DispStatFlags, DisplayStatusRegister}, power_control_register1::PowerControlRegister1, power_control_register2::PowerControlRegister2, vram_control_register::VramControlRegister};

use crate::scheduler::{EventType, Scheduler};

pub mod registers;
pub mod engine_2d;
pub mod engine_3d;

const CYCLES_PER_DOT: usize = 6;
const HBLANK_DOTS: usize = 256 + 8;
const DOTS_PER_LINE: usize = 355;

const NUM_LINES: u16 = 263;

const HEIGHT: u16 = 192;
const WIDTH: u16 = 256;

pub struct GPU {
  pub engine_a: Engine2d<false>,
  pub engine_b: Engine2d<true>,
  pub engine3d: Engine3d,
  pub powcnt1: PowerControlRegister1,
  pub powcnt2: PowerControlRegister2,
  pub vramcnt: [VramControlRegister; 9],
  pub dispstat: [DisplayStatusRegister; 2],
  pub frame_finished: bool,
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
      frame_finished: false
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

      println!("setting hblank to true");
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

    println!("vcount is now {}", self.vcount);

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

  pub fn schedule_next_line(&mut self, scheduler: &mut Scheduler) {
    scheduler.schedule(EventType::NEXT_LINE, CYCLES_PER_DOT * DOTS_PER_LINE);
  }

  fn trigger_vblank(&mut self) {
    for dispstat in &mut self.dispstat {
      println!("setting vblank to true");
      dispstat.flags.insert(DispStatFlags::VBLANK);
    }

    // do some other stuff here
  }

  fn render_line(&mut self) {

  }
}