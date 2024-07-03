use cartridge_control_register::CartridgeControlRegister;
use spicnt::SPICNT;

use crate::util;

pub mod cartridge_control_register;
pub mod spicnt;

pub const CHIP_ID: u32 = 0x1fc2;


#[derive(Debug)]
pub struct Header {
  game_title: String,
  game_code: String,
  maker_code: String,
  unit_code: u8,
  encryption_seed_select: u8,
  device_capacity: u8,
  region: u8,
  rom_version: u8,
  autostart: u8,
  pub arm9_rom_offset: u32,
  pub arm9_entry_address: u32,
  pub arm9_ram_address: u32,
  pub arm9_size: u32,
  pub arm7_rom_offset: u32,
  pub arm7_entry_address: u32,
  pub arm7_ram_address: u32,
  pub arm7_size: u32
}

impl Header {
  pub fn new(rom: &Vec<u8>) -> Self {
    let default_str: &str = "";

    let header = Self {
      game_title: std::str::from_utf8(&rom[0..0xc]).unwrap_or_default().to_string(),
      game_code: std::str::from_utf8(&rom[0xc..0x10]).unwrap_or_default().to_string(),
      maker_code: std::str::from_utf8(&rom[0x10..0x12]).unwrap_or_default().to_string(),
      unit_code: rom[0x12],
      encryption_seed_select: rom[0x13],
      device_capacity: rom[0x14],
      region: rom[0x1d],
      rom_version: rom[0x1e],
      autostart: rom[0x1f],
      arm9_rom_offset: util::read_word(rom, 0x20),
      arm9_entry_address: util::read_word(rom, 0x24),
      arm9_ram_address: util::read_word(rom, 0x28),
      arm9_size: util::read_word(rom, 0x2c),
      arm7_rom_offset: util::read_word(rom, 0x30),
      arm7_entry_address: util::read_word(rom, 0x34),
      arm7_ram_address: util::read_word(rom, 0x38),
      arm7_size: util::read_word(rom, 0x3c)
    };

    println!("{:?}", header);

    header
  }
}

pub struct Cartridge {
  pub rom: Vec<u8>,
  pub control: CartridgeControlRegister,
  pub spicnt: SPICNT,
  pub header: Header
}

impl Cartridge {
  pub fn new(rom: Vec<u8>) -> Self {
    Self {
      control: CartridgeControlRegister::new(),
      spicnt: SPICNT::new(),
      header: Header::new(&rom),
      rom
    }
  }
}