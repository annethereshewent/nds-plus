use engine_2d::Engine2d;
use engine_3d::Engine3d;

pub mod display_control_register;
pub mod engine_2d;
pub mod engine_3d;

pub struct GPU {
  pub engine_a: Engine2d<false>,
  pub engine_b: Engine2d<true>,
  pub engine3d: Engine3d
}

impl GPU {
  pub fn new() -> Self {
    Self {
      engine_a: Engine2d::new(),
      engine_b: Engine2d::new(),
      engine3d: Engine3d::new()
    }
  }
}