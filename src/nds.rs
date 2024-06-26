use crate::cpu::CPU;

pub struct Nds {
  arm9_cpu: CPU<true>,
  arm7_cpu: CPU<false>
}

impl Nds {
  pub fn new() -> Self {
    Self {
      arm9_cpu: CPU::new(),
      arm7_cpu: CPU::new()
    }
  }

  pub fn step(&mut self) {
    self.arm9_cpu.step();
    self.arm9_cpu.step();

    self.arm7_cpu.step();
  }
}