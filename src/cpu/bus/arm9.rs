use super::{Bus, MAIN_MEMORY_SIZE};

impl Bus {
  pub fn arm9_mem_read_32(&mut self, address: u32) -> u32 {
    match address {
      // TODO: possibly make io_read_32 method to handle this
      0x400_0208 => self.arm9.interrupt_master_enable as u32,
      _ => self.arm9_mem_read_16(address) as u32 | ((self.arm9_mem_read_16(address + 2) as u32) << 16)
    }

  }

  pub fn arm9_mem_read_16(&mut self, address: u32) -> u16 {
    match address {
      0x400_0000..=0x4ff_ffff => self.arm9_io_read_16(address),
      _ => self.arm9_mem_read_8(address) as u16 | ((self.arm9_mem_read_8(address + 1) as u16) << 8)
    }
  }

  pub fn arm9_mem_read_8(&mut self, address: u32) -> u8 {

    if address >= 0xffff_0000 {
      return self.arm9.bios9[(address - 0xffff_0000) as usize];
    }

    match address {
      0x200_0000..=0x2ff_ffff => self.main_memory[(address & ((MAIN_MEMORY_SIZE as u32) - 1)) as usize],
      0x400_0000..=0x4ff_ffff => self.arm9_io_read_8(address),
      0x700_0000..=0x7ff_ffff => 0,
      0x800_0000..=0xdff_ffff => {
        0
      }
      _ => {
        panic!("reading from unsupported address: {:X}", address);
      }
    }
  }

  fn arm9_io_read_16(&mut self, address: u32) -> u16 {
    let address = if address & 0xfffe == 0x8000 {
      0x400_0800
    } else {
      address
    };

    match address {
      0x400_0300 => self.arm9.postflg as u16,
      _ => {
        panic!("io register not implemented: {:X}", address);
      }
    }
  }

  fn arm9_io_read_8(&mut self, address: u32) -> u8 {
    let val = self.arm9_io_read_16(address & !(0b1));

    if address & 0b1 == 1 {
      (val >> 8) as u8
    } else {
      (val & 0xff) as u8
    }
  }

  pub fn arm9_mem_write_32(&mut self, address: u32, val: u32) {
    let upper = (val >> 16) as u16;
    let lower = (val & 0xffff) as u16;

    match address {
      0x400_0208 => self.arm7.interrupt_master_enable = val & 0b1 == 1,
      _ => {
        self.arm9_mem_write_16(address, lower);
        self.arm9_mem_write_16(address + 2, upper);
      }
    }
  }

  pub fn arm9_mem_write_16(&mut self, address: u32, val: u16) {
    let upper = (val >> 8) as u8;
    let lower = (val & 0xff) as u8;

    match address {
      0x400_0000..=0x4ff_ffff => self.arm9_io_write_16(address, val),
      _ => {
        self.arm9_mem_write_8(address, lower);
        self.arm9_mem_write_8(address + 1, upper);
      }
    }
  }

  pub fn arm9_mem_write_8(&mut self, address: u32, val: u8) {
    match address {
      0x200_0000..=0x2ff_ffff => self.main_memory[(address & ((MAIN_MEMORY_SIZE as u32) - 1)) as usize] = val,
      0x400_0000..=0x4ff_ffff => self.arm9_io_write_8(address, val),
      0x500_0000..=0x5ff_ffff => self.arm9_mem_write_16(address & 0x3fe, (val as u16) * 0x101),
      _ => {
        panic!("writing to unsupported address: {:X}", address);
      }
    }
  }

  pub fn arm9_io_write_16(&mut self, address: u32, value: u16) {
    let address = if address & 0xfffe == 0x8000 {
      0x400_0800
    } else {
      address
    };

    match address {
      0x400_01a0 => self.cartridge.spicnt.write(value as u32, 0xff00),
      0x400_01a2 => self.cartridge.spicnt.write((value as u32) << 16, 0xff),
      0x400_01a4 => self.cartridge.control.write(value as u32, 0xff00),
      0x400_01a6 => self.cartridge.control.write((value as u32) << 16, 0xff),
      0x400_0208 => self.arm9.interrupt_master_enable = value & 0b1 == 1,
      0x400_0006 => (),
      _ => {
        panic!("io register not implemented: {:X}", address)
      }
    }
  }

  pub fn arm9_io_write_8(&mut self, address: u32, value: u8) {
    let address = if address & 0xffff == 0x8000 {
      0x400_0800
    } else {
      address
    };

    // println!("im being called with address {:X}", address);

    match address {
      _ => {
        let mut temp = self.arm9_mem_read_16(address & !(0b1));

        temp = if address & 0b1 == 1 {
          (temp & 0xff) | (value as u16) << 8
        } else {
          (temp & 0xff00) | value as u16
        };

        self.arm9_mem_write_16(address & !(0b1), temp);
      }
    }

    // todo: implement sound
  }
}