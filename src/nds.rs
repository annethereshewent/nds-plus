use std::{cell::{Cell, RefCell}, rc::Rc};

use crate::cpu::{bus::Bus, CPU};

pub struct Nds {
  arm9_cpu: CPU<true>,
  arm7_cpu: CPU<false>
}

impl Nds {
  pub fn new() -> Self {
    let bus = Rc::new(RefCell::new(Bus::new()));
    Self {
      arm9_cpu: CPU::new(bus.clone()),
      arm7_cpu: CPU::new(bus)
    }
  }

  pub fn step(&mut self) {
    self.arm9_cpu.step();
    self.arm9_cpu.step();

    self.arm7_cpu.step();
  }
}