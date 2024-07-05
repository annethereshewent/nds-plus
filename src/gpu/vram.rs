use std::collections::HashSet;

use super::{registers::vram_control_register::VramControlRegister, BANK_C};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Bank {
  BankA = 0,
  BankB = 1,
  BankC = 2,
  BankD = 3,
  BankE = 4,
  BankF = 5,
  BankG = 6,
  BankH = 7,
  BankI = 8
}

const ENGINE_A_OBJ_BLOCKS: usize = 256 / 16;
const ENGINE_A_BG_BLOCKS: usize = 512 / 16;
const ENGINE_B_BG_BLOCKS: usize = 128 / 16;
const EXTENDED_PALETTE_BLOCKS: usize = 32 / 16;
const ARM7_WRAM_BLOCKS: usize = 2;

const BLOCK_SIZE: usize = 16 * 1024;

pub struct VRam {
  banks: [Vec<u8>; 9],
  lcdc: HashSet<Bank>,
  engine_a_obj: Vec<HashSet<Bank>>,
  engine_a_bg: Vec<HashSet<Bank>>,
  engine_b_bg: Vec<HashSet<Bank>>,
  arm7_wram: Vec<HashSet<Bank>>,
  extended_palette: Vec<HashSet<Bank>>
}

pub const BANK_SIZES: [usize; 9] = [
  128 * 1024,
  128 * 1024,
  128 * 1024,
  128 * 1024,
  64 * 1024,
  16 * 1024,
  16 * 1024,
  32 * 1024,
  16 * 1024
];

impl VRam {
  pub fn new() -> Self {
    Self {
      banks: [
        vec![0; BANK_SIZES[0]],
        vec![0; BANK_SIZES[1]],
        vec![0; BANK_SIZES[2]],
        vec![0; BANK_SIZES[3]],
        vec![0; BANK_SIZES[4]],
        vec![0; BANK_SIZES[5]],
        vec![0; BANK_SIZES[6]],
        vec![0; BANK_SIZES[7]],
        vec![0; BANK_SIZES[8]],
      ],
      lcdc: HashSet::new(),
      engine_a_obj: Self::create_vec(ENGINE_A_OBJ_BLOCKS),
      arm7_wram: Self::create_vec(ARM7_WRAM_BLOCKS),
      extended_palette: Self::create_vec(EXTENDED_PALETTE_BLOCKS),
      engine_a_bg: Self::create_vec(ENGINE_A_BG_BLOCKS),
      engine_b_bg: Self::create_vec(ENGINE_B_BG_BLOCKS)
    }
  }

  pub fn create_vec(size: usize) -> Vec<HashSet<Bank>> {
    let mut vec = Vec::with_capacity(size);

    for _ in 0..size {
      vec.push(HashSet::new());
    }

    vec
  }

  pub fn write_lcdc_bank(&mut self, bank_enum: Bank, address: u32, value: u8) {
    if self.lcdc.contains(&bank_enum) {
      let bank = &mut self.banks[bank_enum as usize];
      let bank_len = bank.len();

      bank[(address as usize) & (bank_len - 1)] = value;
    } else {
      println!("[WARN] bank {:?} not enabled for lcdc", bank_enum);
    }
  }

  pub fn read_lcdc_bank(&mut self, bank_enum: Bank, address: u32) -> u8 {
    if self.lcdc.contains(&bank_enum) {
      let bank = &mut self.banks[bank_enum as usize];

      bank[(address as usize) & (bank.len() - 1)]
    } else {
      println!("[WARN] bank {:?} not enabled for lcdc", bank_enum);
      0
    }
  }

  pub fn read_arm7_wram(&self, address: u32) -> u8 {
    let mut value: u8 = 0;

    let mut index = address as usize & ((2 * BANK_SIZES[BANK_C as usize]) - 1);
    index = index as usize / BANK_SIZES[BANK_C as usize];

    let region = &self.arm7_wram[index];

    let address = address as usize & (BANK_SIZES[BANK_C as usize] - 1);

    for bank_enum in region.into_iter() {
      let bank = &self.banks[*bank_enum as usize];

      value |= bank[address];
    }

    value
  }

  pub fn write_arm7_wram(&mut self, address: u32, val: u8) {
    let mut index = address as usize & ((2 * BANK_SIZES[BANK_C as usize]) - 1);
    index = index as usize / BANK_SIZES[BANK_C as usize];

    let region = &self.arm7_wram[index];

    let address = address as usize & (BANK_SIZES[BANK_C as usize] - 1);

    for bank_enum in region.into_iter() {
      let bank = &mut self.banks[*bank_enum as usize];

      bank[address] = val;
    }
  }

  pub fn get_lcdc_bank(&mut self, block_num: u32) -> &Vec<u8> {
    &self.banks[block_num as usize]
  }

  fn add_mapping(region: &mut Vec<HashSet<Bank>>, bank: Bank, size: usize, offset: usize) {
    for address in (0..size).step_by(BLOCK_SIZE) {
      region[(address + offset) / BLOCK_SIZE].insert(bank);
    }
  }

  fn remove_mapping(region: &mut Vec<HashSet<Bank>>, bank: Bank, size: usize, offset: usize) {
    for address in (0..size).step_by(BLOCK_SIZE) {
      region[(address + offset) / BLOCK_SIZE].remove(&bank);
    }
  }

  pub fn read_engine_a_bg(&self, address: u32) -> u8 {
    let index = address as usize / BLOCK_SIZE;

    let mut value = 0;

    let mask = ENGINE_A_BG_BLOCKS - 1;

    let bank_enums = &self.engine_a_bg[index & mask];
    for bank_enum in bank_enums.iter() {
      let bank = &self.banks[*bank_enum as usize];

      let address = address as usize & (BANK_SIZES[*bank_enum as usize] - 1);

      value |= bank[address as usize];
    }

    value
  }

  pub fn map_bank(&mut self, bank: Bank, vramcnt: &VramControlRegister) {
    let mut size = BANK_SIZES[bank as usize];
    match vramcnt.vram_mst {
      0 => {
        self.lcdc.insert(bank);
      }
      1 => match bank {
        Bank::BankA | Bank::BankB | Bank::BankC | Bank::BankD => {
          let offset = 0x20000 * vramcnt.vram_offset as usize;

          Self::add_mapping(&mut self.engine_a_bg, bank, size, offset);
        }
        Bank::BankE => Self::add_mapping(&mut self.engine_a_bg, bank, size, 0),
        Bank::BankF | Bank::BankG => {
          //(4000h*OFS.0)+(10000h*OFS.1)
          let offset = 0x4000 * ((vramcnt.vram_offset as usize) & 0x1) + 0x10000 * (((vramcnt.vram_offset as usize) >> 1) & 0x1);
          Self::add_mapping(&mut self.engine_a_bg, bank, size, offset);
        }
        Bank::BankH => Self::add_mapping(&mut self.engine_b_bg, bank, size, 0),
        Bank::BankI => Self::add_mapping(&mut self.engine_b_bg, bank, size, 0)
      }
      2 => match bank {
        Bank::BankA | Bank::BankB  => {
          let offset = 0x20000 * ((vramcnt.vram_offset as usize) & 0x1);

          Self::add_mapping(&mut self.engine_a_obj, bank, size, offset);
        }
        Bank::BankE => Self::add_mapping(&mut self.engine_a_obj, bank, size, 0),
        Bank::BankF | Bank::BankG => {
          let offset = 0x4000 * (vramcnt.vram_offset as usize & 0x1) + 0x10000 * (((vramcnt.vram_offset as usize) >> 1) & 0x1);

          Self::add_mapping(&mut self.engine_a_obj, bank, size, offset);
        }
        Bank::BankC | Bank::BankD => {
          let offset = vramcnt.vram_offset;

          self.arm7_wram[offset as usize].insert(bank);
        }
        Bank::BankH => Self::add_mapping(&mut self.extended_palette, bank, size, 0),
        _ => panic!("invalid bank given for mst = {}", vramcnt.vram_mst)
      }
      _ => todo!("mst = {} not yet implemented", vramcnt.vram_mst)
    }
  }

  pub fn unmap_bank(&mut self, bank: Bank, vramcnt: &VramControlRegister) {
    let mut size = BANK_SIZES[bank as usize] as usize;
    match vramcnt.vram_mst {
      0 => {
        self.lcdc.remove(&bank);
      }
      1 => match bank {
        Bank::BankA | Bank::BankB | Bank::BankC | Bank::BankD => {
          let offset = 0x20000 * vramcnt.vram_offset as usize;

          Self::remove_mapping(&mut self.engine_a_bg, bank, size, offset);
        }
        Bank::BankE => Self::add_mapping(&mut self.engine_a_bg, bank, size, 0),
        Bank::BankF | Bank::BankG => {
          //(4000h*OFS.0)+(10000h*OFS.1)
          let offset = 0x4000 * ((vramcnt.vram_offset as usize) & 0x1) + 0x10000 * (((vramcnt.vram_offset as usize) >> 1) & 0x1);
          Self::remove_mapping(&mut self.engine_a_bg, bank, size, offset);
        }
        Bank::BankH => Self::remove_mapping(&mut self.engine_b_bg, bank, size, 0),
        Bank::BankI => Self::remove_mapping(&mut self.engine_b_bg, bank, size, 0)
      }
      2 => match bank {
        Bank::BankA | Bank::BankB  => {
          let offset = 0x20000 * ((vramcnt.vram_offset as usize) & 0x1);

          Self::remove_mapping(&mut self.engine_a_obj, bank, size, offset);
        }
        Bank::BankE => Self::remove_mapping(&mut self.engine_a_obj, bank, size, 0),
        Bank::BankF | Bank::BankG => {
          let offset = 0x4000 * (vramcnt.vram_offset as usize & 0x1) + 0x1000 * (((vramcnt.vram_offset as usize) >> 1) & 0x1);

          Self::remove_mapping(&mut self.engine_a_obj, bank, size, offset);
        }
        Bank::BankC | Bank::BankD => {
          let offset = vramcnt.vram_offset as usize;

          self.arm7_wram[offset].remove(&bank);
        }
        Bank::BankH => Self::remove_mapping(&mut self.extended_palette, bank, size, 0),
        _ => panic!("invalid option given")
      }
      _ => todo!("unimplemented")
    };
  }
}