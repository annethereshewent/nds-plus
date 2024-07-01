use cartridge_control_register::CartridgeControlRegister;
use spicnt::SPICNT;

pub mod cartridge_control_register;
pub mod spicnt;

pub struct Cartridge {
  rom: Vec<u8>,
  chip_id: u32,
  pub control: CartridgeControlRegister,
  pub spicnt: SPICNT
}

impl Cartridge {
  pub fn new(rom: Vec<u8>) -> Self {
    Self {
      rom,
      chip_id: 0,
      control: CartridgeControlRegister::new(),
      spicnt: SPICNT::new()
    }
  }
}