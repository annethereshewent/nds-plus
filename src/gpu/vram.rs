use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::number::Number;

use super::{registers::vram_control_register::VramControlRegister, BANK_C};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize, Debug)]
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

impl Bank {
  pub fn new(value: usize) -> Self {
    use Bank::*;
    match value {
      0 => BankA,
      1 => BankB,
      2 => BankC,
      3 => BankD,
      4 => BankE,
      5 => BankF,
      6 => BankG,
      7 => BankH,
      8 => BankI,
      _ => unreachable!()
    }
  }
}

const ENGINE_A_OBJ_BLOCKS: usize = 256 / 16;
const ENGINE_A_BG_BLOCKS: usize = 512 / 16;
const ENGINE_B_BG_BLOCKS: usize = 128 / 16;
const ENGINE_B_OBJ_BLOCKS: usize = 128 / 16;
const EXTENDED_PALETTE_BLOCKS: usize = 32 / 16;
const ENGINE_A_EXTENDED_OBJ_PALETTE_BLOCKS: usize = 16 / 16;
const ENGINE_B_EXTENDED_OBJ_PALETTE_BLOCKS: usize = 16 / 16;
const TEXTURE_BLOCKS: usize = 512 / 16;
const TEXTURE_PALETTE_BLOCKS: usize = 96 / 16;
const ARM7_WRAM_BLOCKS: usize = 2;

const BLOCK_SIZE: usize = 16 * 1024;

#[derive(Serialize, Deserialize)]
pub struct VRam {
  pub banks: [Vec<u8>; 9],
  pub lcdc: HashSet<Bank>,
  engine_a_obj: Vec<HashSet<Bank>>,
  engine_b_obj: Vec<HashSet<Bank>>,
  engine_a_bg: Vec<HashSet<Bank>>,
  engine_b_bg: Vec<HashSet<Bank>>,
  arm7_wram: Vec<HashSet<Bank>>,
  engine_a_bg_extended_palette: Vec<HashSet<Bank>>,
  engine_b_bg_extended_palette: Vec<HashSet<Bank>>,
  engine_a_obj_extended_palette: Vec<HashSet<Bank>>,
  engine_b_obj_extended_palette: Vec<HashSet<Bank>>,
  textures: Vec<HashSet<Bank>>,
  texture_palette: Vec<HashSet<Bank>>
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
      engine_a_bg_extended_palette: Self::create_vec(EXTENDED_PALETTE_BLOCKS),
      engine_b_bg_extended_palette: Self::create_vec(EXTENDED_PALETTE_BLOCKS),
      engine_a_bg: Self::create_vec(ENGINE_A_BG_BLOCKS),
      engine_b_bg: Self::create_vec(ENGINE_B_BG_BLOCKS),
      engine_b_obj: Self::create_vec(ENGINE_B_OBJ_BLOCKS),
      engine_a_obj_extended_palette: Self::create_vec(ENGINE_A_EXTENDED_OBJ_PALETTE_BLOCKS),
      engine_b_obj_extended_palette: Self::create_vec(ENGINE_B_EXTENDED_OBJ_PALETTE_BLOCKS),
      textures: Self::create_vec(TEXTURE_BLOCKS),
      texture_palette: Self::create_vec(TEXTURE_PALETTE_BLOCKS)
    }
  }

  pub fn create_vec(size: usize) -> Vec<HashSet<Bank>> {
    let mut vec = Vec::with_capacity(size);
    for _ in 0..size {
      vec.push(HashSet::new());
    }

    vec
  }

  pub fn write_lcdc_bank<T: Number>(&mut self, bank_enum: Bank, address: u32, value: T) {
    if self.lcdc.contains(&bank_enum) {
      let bank = &mut self.banks[bank_enum as usize];
      let bank_len = bank.len();

      unsafe { *(&mut bank[(address as usize) & (bank_len - 1)] as *mut u8 as *mut T) = value };
    }
  }

  pub fn read_lcdc_bank<T: Number>(&mut self, bank_enum: Bank, address: u32) -> T {
    if self.lcdc.contains(&bank_enum) {
      let bank = &mut self.banks[bank_enum as usize];

      unsafe { *(&bank[(address as usize) & (bank.len() - 1)] as *const u8 as *const T) }
    } else {
      println!("[WARN] read from bank {:?} not enabled for lcdc", bank_enum);
      num::zero()
    }
  }

  pub fn read_arm7_wram<T: Number>(&self, address: u32) -> T {
    let mut value: T = num::zero();

    let mut index = address as usize & ((2 * BANK_SIZES[BANK_C as usize]) - 1);
    index = index as usize / BANK_SIZES[BANK_C as usize];

    let region = &self.arm7_wram[index];

    let address = address as usize & (BANK_SIZES[BANK_C as usize] - 1);

    for bank_enum in region.into_iter() {
      let bank = &self.banks[*bank_enum as usize];

      value |= unsafe { *(&bank[address] as *const u8 as *const T) };
    }

    value
  }

  pub fn write_arm7_wram<T: Number>(&mut self, address: u32, val: T) {
    let mut index = address as usize & ((2 * BANK_SIZES[BANK_C as usize]) - 1);
    index = index as usize / BANK_SIZES[BANK_C as usize];

    let region = &self.arm7_wram[index];

    let address = address as usize & (BANK_SIZES[BANK_C as usize] - 1);

    for bank_enum in region.into_iter() {
      let bank = &mut self.banks[*bank_enum as usize];

      unsafe { *(&mut bank[address] as *mut u8 as *mut T) = val };
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

  pub fn write_engine_a_obj<T: Number>(&mut self, address: u32, val: T) {
    Self::write_mapping(&mut self.banks, &mut self.engine_a_obj, ENGINE_A_OBJ_BLOCKS - 1, address, val);
  }

  pub fn write_engine_b_obj<T: Number>(&mut self, address: u32, val: T) {
    Self::write_mapping(&mut self.banks, &mut self.engine_b_obj, ENGINE_B_OBJ_BLOCKS - 1, address, val);
  }

  pub fn write_engine_a_bg<T: Number>(&mut self, address: u32, val: T) {
    Self::write_mapping(&mut self.banks,&mut self.engine_a_bg, ENGINE_A_BG_BLOCKS - 1, address, val);
  }

  fn write_mapping<T: Number>(banks: &mut [Vec<u8>], region: &mut Vec<HashSet<Bank>>, mask: usize, address: u32, val: T) {
    let index = address as usize / BLOCK_SIZE;

    let bank_enums = &region[index & mask];

    for bank_enum in bank_enums {
      let bank = &mut banks[*bank_enum as usize];

      let address = address as usize & (BANK_SIZES[*bank_enum as usize] - 1);

      unsafe { *(&mut bank[address] as *mut u8 as *mut T) = val };
    }
  }

  fn read_mapping<T: Number>(banks: &[Vec<u8>], region: &Vec<HashSet<Bank>>, mask: usize, address: u32) -> T {
    let index = address as usize / BLOCK_SIZE;

    let mut value: T = num::zero();

    let bank_enums = &region[index & mask];

    for bank_enum in bank_enums.iter() {
      let bank = &banks[*bank_enum as usize];

      let address = address as usize & (BANK_SIZES[*bank_enum as usize] - 1);

      value |= unsafe { *(&bank[address as usize] as *const u8 as *const T) }
    }

    value
  }

  pub fn read_engine_a_obj<T: Number>(&self, address: u32) -> T {
    Self::read_mapping::<T>(&self.banks, &self.engine_a_obj, ENGINE_A_OBJ_BLOCKS - 1, address)
  }

  pub fn read_engine_b_obj<T: Number>(&self, address: u32) -> T {
    Self::read_mapping::<T>(&self.banks, &self.engine_b_obj, ENGINE_B_OBJ_BLOCKS - 1, address)
  }

  pub fn write_engine_b_bg<T: Number>(&mut self, address: u32, val: T) {
    Self::write_mapping(&mut self.banks, &mut self.engine_b_bg, ENGINE_B_BG_BLOCKS - 1, address, val);
  }

  pub fn read_engine_a_bg<T: Number>(&self, address: u32) -> T {
    Self::read_mapping(&self.banks, &self.engine_a_bg, ENGINE_A_BG_BLOCKS - 1, address)
  }

  pub fn read_engine_a_extended_obj_palette<T: Number>(&self, address: u32) -> T {
    Self::read_mapping(&self.banks, &self.engine_a_obj_extended_palette, ENGINE_A_EXTENDED_OBJ_PALETTE_BLOCKS - 1, address)
  }

  pub fn read_engine_b_extended_obj_palette<T: Number>(&self, address: u32) -> T {
    Self::read_mapping(&self.banks, &self.engine_b_obj_extended_palette, ENGINE_B_EXTENDED_OBJ_PALETTE_BLOCKS - 1, address)
  }

  pub fn read_engine_a_extended_bg_palette<T: Number>(&self, address: u32) -> T {
    Self::read_mapping(&self.banks, &self.engine_a_bg_extended_palette, EXTENDED_PALETTE_BLOCKS - 1, address)
  }

  pub fn read_engine_b_extended_bg_palette<T: Number>(&self, address: u32) -> T {
    Self::read_mapping(&self.banks, &self.engine_b_bg_extended_palette, EXTENDED_PALETTE_BLOCKS - 1, address)
  }

  pub fn read_engine_b_bg<T: Number>(&self, address: u32) -> T {
    Self::read_mapping::<T>(&self.banks, &self.engine_b_bg, ENGINE_B_BG_BLOCKS - 1, address)
  }

  pub fn read_texture_palette<T: Number>(&self, address: u32) -> T {
    Self::read_mapping(&self.banks, &self.texture_palette, TEXTURE_PALETTE_BLOCKS - 1, address)
  }

  pub fn read_texture<T: Number>(&self, address: u32) -> T {
    Self::read_mapping::<T>(&self.banks, &self.textures, TEXTURE_BLOCKS - 1, address)
  }

  pub fn map_bank(&mut self, bank: Bank, vramcnt: &VramControlRegister) {
    let mut size = BANK_SIZES[bank as usize];
    match vramcnt.vram_mst {
      0 => {
        self.lcdc.insert(bank);
      }
      1 => match bank {
        Bank::BankA | Bank::BankB | Bank::BankC | Bank::BankD => {
          let offset = 0x2_0000 * vramcnt.vram_offset as usize;

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
        Bank::BankH => Self::add_mapping(&mut self.engine_b_bg_extended_palette, bank, size, 0),
        Bank::BankI => Self::add_mapping(&mut self.engine_b_obj, bank, size, 0),
      }
      3 => match bank {
        Bank::BankI => {
          size = 0x2000;

          Self::add_mapping(&mut self.engine_b_obj_extended_palette, bank, size, 0);
        }
        Bank::BankA | Bank::BankB | Bank::BankC | Bank::BankD => {
          let offset = 128 * 1024 * vramcnt.vram_offset as usize;

          Self::add_mapping(&mut self.textures, bank, size, offset);
        }
        Bank::BankE => Self::add_mapping(&mut self.texture_palette, bank, size, 0),
        Bank::BankF | Bank::BankG => {
          let index = (vramcnt.vram_offset & 0b1) + ((vramcnt.vram_offset >> 1) & 0b1) * 4;
          let offset = 16 * 1204 * index as usize;

          Self::add_mapping(&mut self.texture_palette, bank, size, offset);
        }
        _ => panic!("invalid bank given for mst = 3")
      }
      4 => match bank {
        Bank::BankC => {
          Self::add_mapping(&mut self.engine_b_bg, bank, size, 0);
        }
        Bank::BankD => {
          Self::add_mapping(&mut self.engine_b_obj, bank, size, 0);
        }
        Bank::BankE => {
          size = 0x8000;

          Self::add_mapping(&mut self.engine_a_bg_extended_palette, bank, size, 0);
        }
        Bank::BankF | Bank::BankG => {
          let offset = match vramcnt.vram_offset {
            0 => 0,
            1 => 16 * 1024,
            _ => panic!("invalid offset given")
          };

          Self::add_mapping(&mut self.engine_a_bg_extended_palette, bank, size, offset);
        }
        _ => panic!("invalid option given for mst = 4")
      }
      5 => match bank {
        Bank::BankF | Bank::BankG => {
          size = 0x2000;

          Self::add_mapping(&mut self.engine_a_obj_extended_palette, bank, size, 0);
        }
        _ => panic!("invalid option given for mst = 5")
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
          let offset = 0x2_0000 * vramcnt.vram_offset as usize;

          Self::remove_mapping(&mut self.engine_a_bg, bank, size, offset);
        }
        Bank::BankE => Self::remove_mapping(&mut self.engine_a_bg, bank, size, 0),
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
          let offset = 0x2_0000 * ((vramcnt.vram_offset as usize) & 0x1);

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
        Bank::BankH => Self::remove_mapping(&mut self.engine_b_bg_extended_palette, bank, size, 0),
        Bank::BankI => Self::remove_mapping(&mut self.engine_b_obj, bank, size, 0),
      }
      3 => match bank {
        Bank::BankI => {
          size = 0x2000;

          Self::remove_mapping(&mut self.engine_b_obj_extended_palette, bank, size, 0);
        }
        Bank::BankA | Bank::BankB | Bank::BankC | Bank::BankD => {
          let offset = 128 * 1024 * vramcnt.vram_offset as usize;

          Self::remove_mapping(&mut self.textures, bank, size, offset);
        }
        Bank::BankE => Self::remove_mapping(&mut self.texture_palette, bank, size, 0),
        Bank::BankF | Bank::BankG => {
          let index = (vramcnt.vram_offset & 0b1) + ((vramcnt.vram_offset >> 1) & 0b1) * 4;
          let offset = 16 * 1204 * index as usize;

          Self::remove_mapping(&mut self.texture_palette, bank, size, offset);
        }
        _ => panic!("invalid bank given for mst = 3")
      }
      4 => match bank {
        Bank::BankC => {
          Self::remove_mapping(&mut self.engine_b_bg, bank, size, 0);
        }
        Bank::BankD => {
          Self::remove_mapping(&mut self.engine_b_obj, bank, size, 0);
        }
        Bank::BankE => {
          size = 0x8000;

          Self::remove_mapping(&mut self.engine_a_bg_extended_palette, bank, size, 0);
        }
        Bank::BankF | Bank::BankG => {
          let offset = match vramcnt.vram_offset {
            0 => 0,
            1 => 16 * 1024,
            _ => panic!("invalid offset given")
          };

          Self::remove_mapping(&mut self.engine_a_bg_extended_palette, bank, size, offset);
        }
        _ => panic!("invalid option given for mst = 4")
      }
      5 => match bank {
        Bank::BankF | Bank::BankG => {
          size = 0x2000;

          Self::remove_mapping(&mut self.engine_a_obj_extended_palette, bank, size, 0);
        }
        _ => panic!("invalid option given for mst = 5")
      }
      _ => todo!("unimplemented")
    };
  }
}