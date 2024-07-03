#[derive(PartialEq, Clone, Copy, Debug)]
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

pub struct VRam {
  banks: [Vec<u8>; 9],
  pub lcdc: Vec<Bank>
}

const BANK_SIZES: [usize; 9] = [
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
      lcdc: Vec::new()
    }
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

  pub fn get_lcdc_bank(&mut self, block_num: u32) -> &Vec<u8> {
    &self.banks[block_num as usize]
  }

  pub fn map_bank(&mut self, bank: Bank, mst: u8) {
    match mst {
      0 => self.lcdc.push(bank),
      _ => todo!("unimplemented")
    }
  }

  pub fn unmap_bank(&mut self, bank: Bank, mst: u8) {
    let region = match mst {
      0 => &mut self.lcdc,
      _ => todo!("unimplemented")
    };

    if let Some(i) = region.iter().position(|&b| b == bank) {
      region.remove(i);
    }
  }
}