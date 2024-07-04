use crate::{cpu::registers::{interrupt_enable_register::InterruptEnableRegister, interrupt_request_register::InterruptRequestRegister}, gpu::registers::power_control_register1::PowerControlRegister1};

use super::{Bus, DTCM_SIZE, ITCM_SIZE, MAIN_MEMORY_SIZE};

impl Bus {
  pub fn arm9_mem_read_32(&mut self, address: u32) -> u32 {
    match address {
      0x400_0000..=0x4ff_ffff => self.arm9_io_read_32(address),
      _ => self.arm9_mem_read_16(address) as u32 | ((self.arm9_mem_read_16(address + 2) as u32) << 16)
    }
  }

  pub fn arm9_io_read_32(&mut self, address: u32) -> u32 {
    match address {
      0x400_0208 => self.arm9.interrupt_master_enable as u32,
      0x400_0210 => self.arm9.interrupt_enable.bits(),
      0x400_0214 => self.arm9.interrupt_request.bits(),
      0x410_0000 => self.receive_from_fifo(true),
      _ => panic!("unsupported io address received: {:X}", address)
    }
  }

  pub fn arm9_mem_read_16(&mut self, address: u32) -> u16 {
    match address {
      0x400_0000..=0x4ff_ffff => self.arm9_io_read_16(address),
      _ => self.arm9_mem_read_8(address) as u16 | ((self.arm9_mem_read_8(address + 1) as u16) << 8)
    }
  }

  pub fn arm9_mem_read_8(&mut self, address: u32) -> u8 {
    let dtcm_ranges = self.arm9.cp15.dtcm_control.get_ranges();
    let itcm_ranges = self.arm9.cp15.itcm_control.get_ranges();

    if dtcm_ranges.contains(&address) {
      let actual_addr = (address - self.arm9.cp15.dtcm_control.base_address()) & (DTCM_SIZE as u32 - 1);

      return self.dtcm[actual_addr as usize];
    } else if itcm_ranges.contains(&address) {
      let actual_addr = (address - self.arm9.cp15.itcm_control.base_address()) & (ITCM_SIZE as u32 - 1);

      return self.itcm[actual_addr as usize];
    }

    if address >= 0xffff_0000 {
      return self.arm9.bios9[(address - 0xffff_0000) as usize];
    }

    match address {
      0x200_0000..=0x2ff_ffff => self.main_memory[(address & ((MAIN_MEMORY_SIZE as u32) - 1)) as usize],
      0x400_0000..=0x4ff_ffff => self.arm9_io_read_8(address),
      0x680_0000..=0x6ff_ffff => self.gpu.read_lcdc(address),
      // 0x700_0000..=0x7ff_ffff => 0,
      // 0x800_0000..=0xdff_ffff => {
      //   0
      // }
      _ => {
        panic!("reading from unsupported address: {:X}", address);
      }
    }
  }

  fn arm9_io_read_16(&mut self, address: u32) -> u16 {
    // not sure if this is needed for the ds....
    // let address = if address & 0xfffe == 0x8000 {
    //   0x400_0800
    // } else {
    //   address
    // };

    match address {
      0x400_0004 => self.gpu.dispstat[1].read(),
      0x400_0300 => self.arm9.postflg as u16,
      0x400_0130 => self.key_input_register.bits(),
      0x400_0180 => self.arm9.ipcsync.read() as u16,
      0x400_0182 => (self.arm9.ipcsync.read() >> 16) as u16,
      0x400_0184 => self.arm9.ipcfifocnt.read(&mut self.arm7.ipcfifocnt.fifo) as u16,
      0x400_0186 => (self.arm9.ipcfifocnt.read(&mut self.arm7.ipcfifocnt.fifo) >> 16) as u16,
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
      0x400_0000..=0x4ff_ffff => self.arm9_io_write_32(address, val),
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
    let dtcm_ranges = self.arm9.cp15.dtcm_control.get_ranges();
    let itcm_ranges = self.arm9.cp15.itcm_control.get_ranges();

    if dtcm_ranges.contains(&address) {
      let actual_addr = (address - self.arm9.cp15.dtcm_control.base_address()) & (DTCM_SIZE as u32 - 1);

      self.dtcm[actual_addr as usize] = val;

      return;
    } else if itcm_ranges.contains(&address) {
      let actual_addr = (address - self.arm9.cp15.itcm_control.base_address()) & (ITCM_SIZE as u32 - 1);

      self.itcm[actual_addr as usize] = val;

      return;
    }

    match address {
      0x200_0000..=0x2ff_ffff => self.main_memory[(address & ((MAIN_MEMORY_SIZE as u32) - 1)) as usize] = val,
      0x400_0000..=0x4ff_ffff => self.arm9_io_write_8(address, val),
      0x500_0000..=0x5ff_ffff => self.arm9_mem_write_16(address & 0x3fe, (val as u16) * 0x101),
      0x680_0000..=0x6ff_ffff => self.gpu.write_lcdc(address, val),
      0x700_0000..=0x700_3fff => self.gpu.engine_a.oam[(address & 0x3ff) as usize] = val,
      0x700_4000..=0x7ff_ffff => self.gpu.engine_b.oam[(address & 0x3ff) as usize] = val,
      0x800_0000..=0x8ff_ffff => (), // todo: fix this
      _ => {
        panic!("writing to unsupported address: {:X}", address);
      }
    }
  }

  pub fn arm9_io_write_32(&mut self, address: u32, value: u32) {
    match address {
      0x400_0000 => self.gpu.engine_a.dispcnt.write(value),
      0x400_0188 => self.send_to_fifo(true, value),
      0x400_0208 => self.arm9.interrupt_master_enable = value & 0b1 == 1,
      0x400_0210 => self.arm9.interrupt_enable = InterruptEnableRegister::from_bits_retain(value),
      0x400_0214 => self.arm9.interrupt_request = InterruptRequestRegister::from_bits_retain(value),
      0x400_0304 => self.gpu.powcnt1 = PowerControlRegister1::from_bits_retain(value),
      _ => panic!("write to unsupported io address: {:X}", address)
    }
  }

  pub fn arm9_io_write_16(&mut self, address: u32, value: u16) {
    // not sure if this is needed for the ds....
    // let address = if address & 0xfffe == 0x8000 {
    //   0x400_0800
    // } else {
    //   address
    // };

    match address {
      0x400_0180 => self.arm9.ipcsync.write(&mut self.arm7.ipcsync, &mut self.arm9.interrupt_request, value),
      0x400_0184 => self.arm9.ipcfifocnt.write(&mut self.arm7.ipcfifocnt.fifo, &mut self.arm7.ipcfifocnt.previous_value, value),
      0x400_01a0 => self.cartridge.spicnt.write(value as u32, 0xff00),
      0x400_01a2 => self.cartridge.spicnt.write((value as u32) << 16, 0xff),
      0x400_01a4 => self.cartridge.control.write(value as u32, 0xff00),
      0x400_01a6 => self.cartridge.control.write((value as u32) << 16, 0xff),
      0x400_0208 => self.arm9.interrupt_master_enable = value & 0b1 == 1,
      // 0x400_0006 => (),
      _ => {
        panic!("io register not implemented: {:X}", address)
      }
    }
  }

  pub fn arm9_io_write_8(&mut self, address: u32, value: u8) {
    // not sure if needed and so on
    // let address = if address & 0xffff == 0x8000 {
    //   0x400_0800
    // } else {
    //   address
    // };

    // println!("im being called with address {:X}", address);

    match address {
      0x400_0240..=0x400_0246 => {
        let offset = address - 0x400_0240;

        self.gpu.write_vramcnt(offset, value);
      }
      0x400_0248 => self.gpu.write_vramcnt(7, value),
      0x400_0249 => self.gpu.write_vramcnt(8, value),
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
