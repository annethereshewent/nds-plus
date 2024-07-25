use std::collections::VecDeque;

use crate::gpu::engine_3d::GeometryCommand;

#[derive(Copy, Clone, PartialEq)]
pub enum GeometryIrq {
  Never = 0 ,
  LessThanHalfFull = 1,
  Empty = 2
}

pub struct GeometryStatusRegister {
  test_busy: bool,
  box_test_result: bool,
  matrix_stack_busy: bool,
  matrix_stack_error: bool,
  geometry_engine_busy: bool,
  geometry_irq: GeometryIrq
}

impl GeometryStatusRegister {
  pub fn new() -> Self {
    Self {
      test_busy: false,
      box_test_result: false,
      matrix_stack_busy: false,
      matrix_stack_error: false,
      geometry_engine_busy: false,
      geometry_irq: GeometryIrq::Never
    }
  }

  pub fn read(&self, position_stack_level: u32, projection_stack_level: u32, fifo: &VecDeque<GeometryCommand>) -> u32 {
    (self.test_busy as u32) |
      (self.box_test_result as u32) << 1 |
      (position_stack_level & 0x1f) << 8 |
      projection_stack_level << 13 |
      (self.matrix_stack_busy as u32) << 14 |
      (self.matrix_stack_error as u32) << 15 |
      (fifo.len() as u32) << 16 |
      ((fifo.len() < fifo.capacity() / 2) as u32) << 25 |
      (fifo.is_empty() as u32) << 26 |
      (self.geometry_engine_busy as u32) << 27 |
      (self.geometry_irq as u32) << 30
  }

  pub fn write(&mut self, value: u32) {
    self.geometry_irq = match (value >> 30) & 0x3 {
      0 => GeometryIrq::Never,
      1 => GeometryIrq::LessThanHalfFull,
      2 => GeometryIrq::Empty,
      _ => panic!("invalid value given for geometry irq: {}", (value >> 30) & 0x3)
    };

    if value >> 15 & 0b1 == 1 {
      // todo: reset matrix stack levels here
      self.matrix_stack_error = false;
    }

    // todo: interrupts here
  }
}