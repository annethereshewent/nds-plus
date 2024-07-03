pub struct IPCSyncRegister {
  pub data_output: u32,
  pub data_input: u32,
  pub send_irq: bool,
  pub irq_enable: bool
}

impl IPCSyncRegister {
  pub fn new() -> Self {
    Self {
      data_input: 0,
      data_output: 0,
      send_irq: false,
      irq_enable: false
    }
  }

  pub fn read(&self) -> u32 {
    self.data_input | (self.data_output) << 8 | (self.irq_enable as u32) << 14
  }

  pub fn write(&mut self, other: &mut IPCSyncRegister, value: u16) {
    self.data_output = ((value >> 8) & 0xf) as u32;
    other.data_input = self.data_output;


    self.send_irq = (value >> 13) & 0b1 == 1;
    self.irq_enable = (value >> 14) & 0b1 == 1;

    if other.irq_enable && self.send_irq {
      // send IRQ here TODO
    }
  }
}