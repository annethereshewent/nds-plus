use crate::cpu::registers::{interrupt_enable_register::InterruptEnableRegister, interrupt_request_register::InterruptRequestRegister, ipc_fifo_control_register::FIFO_CAPACITY};

use super::{Bus, MAIN_MEMORY_SIZE, WRAM_SIZE};

impl Bus {
  pub fn arm7_mem_read_32(&mut self, address: u32) -> u32 {
    match address {
      0x400_0000..=0x4ff_ffff => self.arm7_io_read_32(address),
      _ => self.arm7_mem_read_16(address) as u32 | ((self.arm7_mem_read_16(address + 2) as u32) << 16)
    }
  }

  pub fn arm7_io_write_32(&mut self, address: u32, val: u32) {
    match address {
      0x400_0208 => self.arm7.interrupt_master_enable = val != 0,
      0x400_0210 => self.arm7.interrupt_enable = InterruptEnableRegister::from_bits_retain(val),
      0x400_0214 => self.arm7.interrupt_request = InterruptRequestRegister::from_bits_retain(val),
      0x400_0188 => self.send_to_fifo(false, val),
      _ => panic!("write to unsupported address: {:X}", address)
    }
  }

  pub fn arm7_mem_read_16(&mut self, address: u32) -> u16 {
    match address {
      0x400_0000..=0x4ff_ffff => self.arm7_io_read_16(address),
      _ => self.arm7_mem_read_8(address) as u16 | ((self.arm7_mem_read_8(address + 1) as u16) << 8)
    }
  }

  pub fn arm7_mem_read_8(&mut self, address: u32) -> u8 {
    let bios_len = self.arm7.bios7.len() as u32;

    if (0..bios_len).contains(&address) {
      return self.arm7.bios7[address as usize];
    }

    match address {
      0x200_0000..=0x2ff_ffff => self.main_memory[(address & ((MAIN_MEMORY_SIZE as u32) - 1)) as usize],
      0x300_0000..=0x37f_ffff => {
        if self.wramcnt.arm7_size == 0 {
          // per martin koth: "De-allocation (0K) is a special case: At the ARM9-side,
          // the WRAM area is then empty (containing undefined data). At the ARM7-side,
          // the WRAM area is then containing mirrors of the 64KB ARM7-WRAM
          // (the memory at 3800000h and up)."
          return self.arm7.wram[(address & ((WRAM_SIZE as u32) - 1)) as usize];
        }

        let actual_addr = (address & (self.wramcnt.arm7_size - 1)) + self.wramcnt.arm7_offset;

        self.shared_wram[actual_addr as usize]
      }
      0x380_0000..=0x3ff_ffff => self.arm7.wram[(address & ((WRAM_SIZE as u32) - 1)) as usize], // FIX THIS
      0x400_0000..=0x4ff_ffff => self.arm7_io_read_8(address),
      0x600_0000..=0x6ff_ffff => self.gpu.read_arm7_wram(address),
      0x700_0000..=0x7ff_ffff => 0,
      0x800_0000..=0x9ff_ffff => self.read_gba_rom(address, false),
      _ => {
        panic!("reading from unsupported address: {:X}", address);
      }
    }
  }

  fn arm7_io_read_32(&mut self, address: u32) -> u32 {
    match address {
      0x400_0180 => self.arm7.ipcsync.read(),
      0x400_0208 => self.arm7.interrupt_master_enable as u32,
      0x400_0210 => self.arm7.interrupt_enable.bits(),
      0x400_0214 => self.arm7.interrupt_request.bits(),
      0x410_0000 => self.receive_from_fifo(false),
      _ => panic!("unhandled io read to address {:x}", address)
    }
  }

  fn arm7_io_read_16(&mut self, address: u32) -> u16 {
    // let address = if address & 0xfffe == 0x8000 {
    //   0x400_0800
    // } else {
    //   address
    // };

    match address {
      0x400_0004 => self.gpu.dispstat[0].read(),
      0x400_0100 => self.arm7.timers.t[0].read_timer_value(&self.scheduler),
      0x400_0134 => 0, // RCNT register, some kind of debug thing idk
      0x400_0180 => self.arm7.ipcsync.read() as u16,
      0x400_0184 => self.arm7.ipcfifocnt.read(&mut self.arm9.ipcfifocnt.fifo) as u16,
      0x400_01c0 => self.arm7.spicnt.read(),
      0x400_01c2 => self.read_spi_data() as u16,
      0x400_0208 => self.arm7.interrupt_master_enable as u16,
      0x400_0240 => {
        // special case where it's reading from 2 different registers
        self.gpu.get_arm7_vram_stat() as u16 | ((self.wramcnt.read() & 0x3) as u16) << 8
      }
      0x400_0300 => self.arm7.postflg as u16,
      0x400_0400..=0x400_051c => {
        println!("ignoring reads from spu registers");
        0
      }
      _ => {
        panic!("io register not implemented: {:X}", address);
      }
    }
  }

  fn arm7_io_read_8(&mut self, address: u32) -> u8 {
    let val = self.arm7_io_read_16(address & !(0b1));

    if address & 0b1 == 1 {
      (val >> 8) as u8
    } else {
      (val & 0xff) as u8
    }
  }

  pub fn arm7_mem_write_32(&mut self, address: u32, val: u32) {
    let upper = (val >> 16) as u16;
    let lower = (val & 0xffff) as u16;

    match address {
      0x400_0000..=0x4ff_ffff => self.arm7_io_write_32(address, val),
      _ => {
        self.arm7_mem_write_16(address, lower);
        self.arm7_mem_write_16(address + 2, upper);
      }
    }
  }

  pub fn arm7_mem_write_16(&mut self, address: u32, val: u16) {
    let upper = (val >> 8) as u8;
    let lower = (val & 0xff) as u8;

    match address {
      0x400_0000..=0x4ff_ffff => self.arm7_io_write_16(address, val),
      _ => {
        self.arm7_mem_write_8(address, lower);
        self.arm7_mem_write_8(address + 1, upper);
      }
    }
  }

  pub fn arm7_mem_write_8(&mut self, address: u32, val: u8) {
    match address {
      0x200_0000..=0x2ff_ffff => self.main_memory[(address & ((MAIN_MEMORY_SIZE as u32) - 1)) as usize] = val,
      0x300_0000..=0x37f_ffff => {
        if self.wramcnt.arm7_size == 0 {
          panic!("accessing shared ram when bank is currently inaccessable");
        }

        let actual_addr = address & (self.wramcnt.arm7_size - 1) + self.wramcnt.arm7_offset;

        self.shared_wram[actual_addr as usize] = val;
      }
      0x380_0000..=0x3ff_ffff => self.arm7.wram[(address & ((WRAM_SIZE as u32) - 1)) as usize] = val,
      0x400_0000..=0x4ff_ffff => self.arm7_io_write_8(address, val),
      0x500_0000..=0x5ff_ffff => self.arm7_mem_write_16(address & 0x3fe, (val as u16) * 0x101),
      0x600_0000..=0x6ff_ffff => self.gpu.vram.write_arm7_wram(address, val),
      0x800_0000..=0x8ff_ffff => (),
      _ => {
        panic!("writing to unsupported address: {:X}", address);
      }
    }
  }

  pub fn arm7_io_write_16(&mut self, address: u32, value: u16) {
    let address = if address & 0xfffe == 0x8000 {
      0x400_0800
    } else {
      address
    };

    match address {
      // 0x400_0006 => (),
      0x400_0004 => self.gpu.dispstat[0].write(value),
      0x400_0100 => self.arm7.timers.t[0].reload_timer_value(value),
      0x400_0102 => self.arm7.timers.t[0].write_timer_control(value, &mut self.scheduler),
      0x400_0104 => self.arm7.timers.t[0].reload_timer_value(value),
      0x400_0134 => (), // RCNT
      0x400_0106 => self.arm7.timers.t[1].write_timer_control(value, &mut self.scheduler),
      0x400_0180 => self.arm7.ipcsync.write(&mut self.arm9.ipcsync, &mut self.arm7.interrupt_request, value),
      0x400_0184 => self.arm7.ipcfifocnt.write(&mut self.arm7.interrupt_request,&mut self.arm9.ipcfifocnt.fifo,  value),
      0x400_01c0 => self.arm7.spicnt.write(value),
      0x400_01c2 => self.write_spi_data(value as u8), // upper 8 bits are always ignored, even in bugged spi 16 bit mode. per the docs
      0x400_0208 => self.arm7.interrupt_master_enable = value != 0,
      0x400_0210 => {
        let mut value = self.arm7.interrupt_enable.bits() & 0xffff0000;

        value |= value as u32;

        self.arm7.interrupt_enable = InterruptEnableRegister::from_bits_retain(value);
      }
      0x400_0212 => {
        let mut value = self.arm7.interrupt_enable.bits() & 0xffff;

        value |= (value as u32) << 16;

        self.arm7.interrupt_enable = InterruptEnableRegister::from_bits_retain(value);
      }
      0x400_0214 => {
        let mut value = self.arm7.interrupt_request.bits() & 0xffff0000;

        value |= value as u32;

        self.arm7.interrupt_request = InterruptRequestRegister::from_bits_retain(value);
      }
      0x400_0216 => {
        let mut value = self.arm7.interrupt_request.bits() & 0xffff;

        value |= (value as u32) << 16;

        self.arm7.interrupt_request = InterruptRequestRegister::from_bits_retain(value);
      }
      0x400_0400..=0x400_051c => println!("ignoring writes to sound registers"),
      _ => {
        panic!("io register not implemented: {:X}", address)
      }
    }
  }

  pub fn arm7_io_write_8(&mut self, address: u32, value: u8) {
    let address = if address & 0xffff == 0x8000 {
      0x400_0800
    } else {
      address
    };

    // println!("im being called with address {:X}", address);

    match address {
      _ => {
        let mut temp = self.arm7_mem_read_16(address & !(0b1));

        temp = if address & 0b1 == 1 {
          (temp & 0xff) | (value as u16) << 8
        } else {
          (temp & 0xff00) | value as u16
        };

        self.arm7_mem_write_16(address & !(0b1), temp);
      }
    }
    // todo: implement sound
  }

}