use engine_2d::Engine2d;
use engine_3d::Engine3d;
use registers::{display_status_register::DisplayStatusRegister, power_control_register1::PowerControlRegister1, power_control_register2::PowerControlRegister2, vram_control_register::VramControlRegister};

pub mod registers;
pub mod engine_2d;
pub mod engine_3d;

pub struct GPU {
  pub engine_a: Engine2d<false>,
  pub engine_b: Engine2d<true>,
  pub engine3d: Engine3d,
  pub powcnt1: PowerControlRegister1,
  pub powcnt2: PowerControlRegister2,
  pub vramcnt: [VramControlRegister; 9],
  pub dispstat: [DisplayStatusRegister; 2]
}

impl GPU {
  pub fn new() -> Self {
    let mut vramcnt: Vec<VramControlRegister> = Vec::new();

    for i in 0..9 {
      vramcnt.push(VramControlRegister::new(i));
    }

    Self {
      engine_a: Engine2d::new(),
      engine_b: Engine2d::new(),
      engine3d: Engine3d::new(),
      powcnt1: PowerControlRegister1::from_bits_retain(0),
      powcnt2: PowerControlRegister2::from_bits_retain(0),
      vramcnt: vramcnt.try_into().unwrap(),
      dispstat: [DisplayStatusRegister::new(), DisplayStatusRegister::new()]
    }
  }
}