use std::{cell::RefCell, rc::Rc};

use crate::cpu::{bus::Bus, CPU};

pub struct Nds {
  arm9_cpu: CPU<true>,
  arm7_cpu: CPU<false>
}

impl Nds {
  pub fn new(firmware_bytes: Vec<u8>, bios7_bytes: Vec<u8>, bios9_bytes: Vec<u8>, rom_bytes: Vec<u8>) -> Self {
    let bus = Rc::new(RefCell::new(Bus::new(firmware_bytes, bios7_bytes, bios9_bytes, rom_bytes)));
    let mut nds = Self {
      arm9_cpu: CPU::new(bus.clone()),
      arm7_cpu: CPU::new(bus),
    };

    nds.arm7_cpu.reload_pipeline32();
    nds.arm9_cpu.reload_pipeline32();

    nds
  }

  pub fn step(&mut self) {
    self.arm9_cpu.step();
    self.arm9_cpu.step();

    self.arm7_cpu.step();
  }
}