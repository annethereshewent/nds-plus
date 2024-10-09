use crate::{
  apu::BitLength,
  cpu::registers::{
    external_memory::AccessRights,
    interrupt_enable_register::InterruptEnableRegister,
    interrupt_request_register::InterruptRequestRegister
  },
  gpu::registers::power_control_register2::PowerControlRegister2
};

use crate::number::Number;

use super::{Bus, MAIN_MEMORY_SIZE, WRAM_SIZE};

impl Bus {
  pub fn arm7_mem_read_32(&mut self, address: u32) -> u32 {
    match address {
      0x400_0000..=0x4ff_ffff => self.arm7_io_read_32(address),
      _ => self.arm7_mem_read::<u32>(address)
    }
  }

  pub fn arm7_io_write_32(&mut self, address: u32, val: u32) {
    match address {
      0x400_00b0..=0x400_00ba => self.arm7.dma.write(0, (address - 0x400_00b0) as usize, val, None, &mut self.scheduler),
      0x400_00bc..=0x400_00c6 => self.arm7.dma.write(1, (address - 0x400_00bc) as usize, val, None, &mut self.scheduler),
      0x400_00c8..=0x400_00d2 => self.arm7.dma.write(2, (address - 0x400_00c8) as usize, val, None, &mut self.scheduler),
      0x400_00d4..=0x400_00de => self.arm7.dma.write(3, (address - 0x400_00d4) as usize, val, None, &mut self.scheduler),
      0x400_0100..=0x400_010e => {
        self.arm7_io_write_16(address, val as u16);
        self.arm7_io_write_16(address, (val >> 16) as u16);
      }
      0x400_0188 => self.send_to_fifo(false, val),
      0x400_01a4 => {
        self.arm7_io_write_16(address, val as u16);
        self.arm7_io_write_16(address + 2, (val >> 16) as u16);
      }
      0x400_01b0..=0x400_01ba => println!("ignoring writes to key2 encryption seeds"),
      0x400_0208 => self.arm7.interrupt_master_enable = val & 0b1 != 0,
      0x400_0210 => self.arm7.interrupt_enable = InterruptEnableRegister::from_bits_retain(val),
      0x400_0214 => self.arm7.interrupt_request = InterruptRequestRegister::from_bits_retain(self.arm7.interrupt_request.bits() & !val),
      0x400_0308 => (), // ignore writes to biosprot IO
      0x400_0400..=0x400_04ff => self.arm7.apu.write_channels(address, val, &mut self.scheduler, BitLength::Bit32),
      0x400_0510 => self.arm7.apu.sndcapcnt[0].write_destination(val, None),
      0x400_0514 => self.arm7.apu.sndcapcnt[0].write_length(val as u16, None),
      0x400_0518 => self.arm7.apu.sndcapcnt[1].write_destination(val, None),
      0x400_0120 => (),
      0x400_0128 => (),
      0x400_051c => self.arm7.apu.sndcapcnt[1].write_length(val as u16, None),
      _ => println!("[WARN] write to unsupported address: {:X}", address)
    }
  }

  pub fn arm7_mem_read_16(&mut self, address: u32) -> u16 {
    match address {
      0x400_0000..=0x4ff_ffff => self.arm7_io_read_16(address),
      _ => self.arm7_mem_read::<u16>(address)
    }
  }

  pub fn arm7_mem_read<T: Number>(&mut self, address: u32) -> T {
    let bios_len = self.arm7.bios7.len() as u32;

    if (0..bios_len).contains(&address) {
      return unsafe { *(&self.arm7.bios7[address as usize] as *const u8 as *const T) };
    }

    match address {
      0x200_0000..=0x2ff_ffff => {
        unsafe { *(&self.main_memory[(address & ((MAIN_MEMORY_SIZE as u32) - 1)) as usize] as *const u8 as *const T) }
      }
      0x300_0000..=0x37f_ffff => {
        if self.wramcnt.arm7_size == 0 {
          // per martin korth: "De-allocation (0K) is a special case: At the ARM9-side,
          // the WRAM area is then empty (containing undefined data). At the ARM7-side,
          // the WRAM area is then containing mirrors of the 64KB ARM7-WRAM
          // (the memory at 3800000h and up)."
          return unsafe { *(&self.arm7.wram[(address & ((WRAM_SIZE as u32) - 1)) as usize] as *const u8 as *const T) };
        }

        let actual_addr = (address & (self.wramcnt.arm7_size - 1)) + self.wramcnt.arm7_offset;

        unsafe { *(&self.shared_wram[actual_addr as usize] as *const u8 as *const T) }
      }
      0x380_0000..=0x3ff_ffff => {
        unsafe { *(&self.arm7.wram[(address & ((WRAM_SIZE as u32) - 1)) as usize] as *const u8 as *const T) }
      }
      0x600_0000..=0x6ff_ffff => self.gpu.read_arm7_wram(address),
      0x700_0000..=0x7ff_ffff => num::zero(),
      0x800_0000..=0x9ff_ffff => self.read_gba_rom(address, false),
      _ => {
        panic!("reading from unsupported address: {:X}", address);
      }
    }
  }

  pub fn arm7_mem_read_8(&mut self, address: u32) -> u8 {
    match address {
      0x400_0000..=0x4ff_ffff => self.arm7_io_read_8(address),
      _ => self.arm7_mem_read::<u8>(address)
    }
  }

  fn arm7_io_read_32(&mut self, address: u32) -> u32 {
    // println!("reading from arm7 io address {:x}", address);
    match address {
      0x400_0180 => self.arm7.ipcsync.read(),
      0x400_01a0..=0x400_01ac => self.arm7_io_read_16(address) as u32 | (self.arm7_io_read_16(address + 2) as u32) << 16,
      0x400_00b0..=0x400_00ba => self.arm7.dma.read(0, (address - 0x400_00b0) as usize),
      0x400_00bc..=0x400_00c6 => self.arm7.dma.read(1, (address - 0x400_00bc) as usize),
      0x400_00c8..=0x400_00d2 => self.arm7.dma.read(2, (address - 0x400_00c8) as usize),
      0x400_00d4..=0x400_00de => self.arm7.dma.read(3, (address - 0x400_00d4) as usize),
      0x400_01c0 => self.arm7_io_read_16(address) as u32 | (self.arm7_io_read_16(address + 2) as u32) << 16,
      0x400_0208 => self.arm7.interrupt_master_enable as u32,
      0x400_0210 => self.arm7.interrupt_enable.bits(),
      0x400_0214 => self.arm7.interrupt_request.bits(),
      0x400_0400..=0x400_04ff => self.arm7_io_read_16(address) as u32 | (self.arm7_io_read_16(address + 2) as u32) << 16,
      0x400_4000..=0x400_4d08 => 0,
      0x410_0000 => self.receive_from_fifo(false),
      0x410_0010 => self.cartridge.read_gamecard_bus(&mut self.scheduler, self.exmem.nds_access_rights == AccessRights::Arm7, false),
      _ => {
        println!("[WARN] unhandled io read to address {:x}", address);
        0
      }
    }
  }

  fn arm7_io_read_16(&mut self, address: u32) -> u16 {
    // println!("reading from arm7 io address {:x}", address);
    // let address = if address & 0xfffe == 0x8000 {
    //   0x400_0800
    // } else {
    //   address
    // };

    match address {
      0x400_0004 => self.gpu.dispstat[0].read(),
      0x400_0006 => self.gpu.vcount,
      0x400_0100 => self.arm7.timers.t[0].read_timer_value(&self.scheduler),
      0x400_0102 => self.arm7.timers.t[0].timer_ctl.bits(),
      0x400_0104 => self.arm7.timers.t[1].read_timer_value(&self.scheduler),
      0x400_0106 => self.arm7.timers.t[1].timer_ctl.bits(),
      0x400_0108 => self.arm7.timers.t[2].read_timer_value(&self.scheduler),
      0x400_010a => self.arm7.timers.t[2].timer_ctl.bits(),
      0x400_010c => self.arm7.timers.t[3].read_timer_value(&self.scheduler),
      0x400_010e => self.arm7.timers.t[3].timer_ctl.bits(),
      0x400_0130 => self.key_input_register.bits(),
      0x400_0134 => 0, // RCNT register, some kind of debug thing idk
      0x400_0136 => self.arm7.extkeyin.bits(),
      0x400_0138 => self.arm7.rtc.read() as u16,
      0x400_0180 => self.arm7.ipcsync.read() as u16,
      0x400_0184 => self.arm7.ipcfifocnt.read(&mut self.arm9.ipcfifocnt.fifo) as u16,
      0x400_01a0 => self.cartridge.spicnt.read(self.exmem.nds_access_rights == AccessRights::Arm7),
      0x400_01a2 => self.cartridge.read_spidata(self.exmem.nds_access_rights == AccessRights::Arm7) as u16,
      0x400_01a4 => self.cartridge.control.read(self.exmem.nds_access_rights == AccessRights::Arm7) as u16,
      0x400_01a6 => (self.cartridge.control.read(self.exmem.nds_access_rights == AccessRights::Arm7) >> 16) as u16,
      0x400_01a8 => self.cartridge.command[0] as u16 | (self.cartridge.command[1] as u16) << 8,
      0x400_01aa => self.cartridge.command[2] as u16 | (self.cartridge.command[3] as u16) << 8,
      0x400_01ac => self.cartridge.command[4] as u16 | (self.cartridge.command[5] as u16) << 8,
      0x400_01ae => self.cartridge.command[6] as u16 | (self.cartridge.command[7] as u16) << 8,
      0x400_00b0..=0x400_00ba => {
        let actual_addr = address & !(0b11);
        let value = self.arm7.dma.read(0, (actual_addr - 0x400_00b0) as usize);

        if address & 0x3 != 2 {
          value as u16
        } else {
          (value >> 16) as u16
        }
      }
      0x400_00bc..=0x400_00c6 => {
        let actual_addr = address & !(0b11);
        let value = self.arm7.dma.read(1, (actual_addr - 0x400_00bc) as usize);

        if address & 0x3 != 2 {
          value as u16
        } else {
          (value >> 16) as u16
        }
      }
      0x400_00c8..=0x400_00d2 => {
        let actual_addr = address & !(0b11);
        let value = self.arm7.dma.read(2, (actual_addr - 0x400_00c8) as usize);

        if address & 0x3 != 2 {
          value as u16
        } else {
          (value >> 16) as u16
        }
      }
      0x400_00d4..=0x400_00de => {
        let actual_addr = address & !(0b11);
        let value = self.arm7.dma.read(3, (actual_addr - 0x400_00d4) as usize);

        if address & 0x3 != 2 {
          value as u16
        } else {
          (value >> 16) as u16
        }
      }
      0x400_01c0 => self.arm7.spicnt.read(),
      0x400_01c2 => self.read_spi_data() as u16,
      0x400_0204 => self.exmem.read(false),
      0x400_0208 => self.arm7.interrupt_master_enable as u16,
      0x400_0240 => {
        // special case where it's reading from 2 different registers
        self.gpu.get_arm7_vram_stat() as u16 | ((self.wramcnt.read() & 0x3) as u16) << 8
      }
      0x400_0300 => self.arm7.postflg as u16,
      0x400_0304 => self.gpu.powcnt2.bits(),
      0x400_0400..=0x400_04ff => self.arm7.apu.read_channels(address),
      0x400_0500 => self.arm7.apu.soundcnt.read(),
      0x400_0504 => self.arm7.apu.sound_bias,
      0x400_0508 => self.arm7.apu.sndcapcnt[0].read() as u16 | (self.arm7.apu.sndcapcnt[1].read() as u16) << 8,
      0x400_4000..=0x400_4d08 => 0,
      0x480_4000..=0x480_5fff => 0, // more wifi register stuff
      0x480_8000..=0x480_8fff => 0, // TODO: Wifi registers. might need to implement *something* because iirc some games will get stuck in infinite loop
      _ => {
        println!("[WARN] read from io register not implemented: {:X}", address);
        0
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
    match address {
      0x400_0000..=0x4ff_ffff => self.arm7_io_write_32(address, val),
      _ => self.arm7_mem_write::<u32>(address, val)
    }
  }

  pub fn arm7_mem_write_16(&mut self, address: u32, val: u16) {
    match address {
      0x400_0000..=0x4ff_ffff => self.arm7_io_write_16(address, val),
      _ => self.arm7_mem_write::<u16>(address, val)
    }
  }

  pub fn arm7_mem_write<T: Number>(&mut self, address: u32, val: T) {
    if (0..self.arm7.bios7.len()).contains(&(address as usize)) {
      return;
    }

    match address {
      0x200_0000..=0x2ff_ffff => {
        unsafe { *(&mut self.main_memory[(address & ((MAIN_MEMORY_SIZE as u32) - 1)) as usize] as *mut u8 as *mut T) = val }
      }
      0x300_0000..=0x37f_ffff => {
        if self.wramcnt.arm7_size == 0 {
          panic!("accessing shared ram when bank is currently inaccessible");
        }

        let actual_addr = address & (self.wramcnt.arm7_size - 1) + self.wramcnt.arm7_offset;

        unsafe { *(&mut self.shared_wram[actual_addr as usize] as *mut u8 as *mut T) = val };
      }
      0x380_0000..=0x3ff_ffff => {
        unsafe { *(&mut self.arm7.wram[(address & ((WRAM_SIZE as u32) - 1)) as usize] as *mut u8 as *mut T) = val }
      }
      0x600_0000..=0x6ff_ffff => self.gpu.vram.write_arm7_wram(address, val),
      0x800_0000..=0x8ff_ffff => (),
      _ => {
        panic!("writing to unsupported address: {:X}", address);
      }
    }
  }


  pub fn arm7_mem_write_8(&mut self, address: u32, val: u8) {
    if (0..self.arm7.bios7.len()).contains(&(address as usize)) {
      return;
    }

    match address {
      0x400_0000..=0x4ff_ffff => self.arm7_io_write_8(address, val),
      _ => self.arm7_mem_write::<u8>(address, val)
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
      0x400_00b0..=0x400_00ba => {
        let actual_addr = address & !(0x3);

        if address & 0x3 != 2 {
          self.arm7.dma.write(0, (actual_addr - 0x400_00b0) as usize,  value as u32, Some(0xffff0000), &mut self.scheduler);
        } else {
          self.arm7.dma.write(0, (actual_addr - 0x400_00b0) as usize,  (value as u32) << 16, Some(0xffff), &mut self.scheduler);
        }
      }
      0x400_00bc..=0x400_00c6 => {
        let actual_addr = address & !(0x3);

        if address & 0x3 != 2 {
          self.arm7.dma.write(1, (actual_addr - 0x400_00bc) as usize, value as u32, Some(0xffff0000), &mut self.scheduler);
        } else {
          self.arm7.dma.write(1, (actual_addr - 0x400_00bc) as usize, (value as u32) << 16, Some(0xffff), &mut self.scheduler);
        }
      }
      0x400_00c8..=0x400_00d2 => {
        let actual_addr = address & !(0x3);

        if address & 0x3 != 2 {
          self.arm7.dma.write(2, (actual_addr - 0x400_00c8) as usize, value as u32, Some(0xffff0000), &mut self.scheduler);
        } else {
          self.arm7.dma.write(2, (actual_addr - 0x400_00c8) as usize, (value as u32) << 16, Some(0xffff), &mut self.scheduler);
        }
      }
      0x400_00d4..=0x400_00de => {
        let actual_addr = address & !(0x3);

        if address & 0x3 != 2 {
          self.arm7.dma.write(3, (actual_addr - 0x400_00d4) as usize, value as u32, Some(0xffff0000), &mut self.scheduler);
        } else {
          self.arm7.dma.write(3, (actual_addr - 0x400_00d4) as usize, (value as u32) << 16, Some(0xffff), &mut self.scheduler);
        }
      }
      0x400_0100 => self.arm7.timers.t[0].reload_timer_value(value),
      0x400_0102 => self.arm7.timers.t[0].write_timer_control(value, &mut self.scheduler),
      0x400_0104 => self.arm7.timers.t[1].reload_timer_value(value),
      0x400_0106 => self.arm7.timers.t[1].write_timer_control(value, &mut self.scheduler),
      0x400_0108 => self.arm7.timers.t[2].reload_timer_value(value),
      0x400_010a => self.arm7.timers.t[2].write_timer_control(value, &mut self.scheduler),
      0x400_010c => self.arm7.timers.t[3].reload_timer_value(value),
      0x400_010e => self.arm7.timers.t[3].write_timer_control(value, &mut self.scheduler),
      0x400_0128 => (), // debug register
      0x400_0134 => (), // RCNT
      0x400_0138 => self.arm7.rtc.write(value),
      0x400_0180 => self.arm7.ipcsync.write(&mut self.arm9.ipcsync, &mut self.arm9.interrupt_request, value),
      0x400_0184 => self.arm7.ipcfifocnt.write(&mut self.arm7.interrupt_request,&mut self.arm9.ipcfifocnt.fifo,value),
      0x400_01a0 => self.cartridge.spicnt.write(value, self.exmem.nds_access_rights == AccessRights::Arm7, None),
      0x400_01a2 => self.cartridge.write_spidata(value as u8, self.exmem.nds_access_rights == AccessRights::Arm7), // despite being 16-bit, only the first 8 bits matter
      0x400_01a4 => self.cartridge.write_control(value as u32, Some(0xffff0000), &mut self.scheduler, false, self.exmem.nds_access_rights == AccessRights::Arm7),
      0x400_01a6 => self.cartridge.write_control((value as u32) << 16, Some(0xffff), &mut self.scheduler, false, self.exmem.nds_access_rights == AccessRights::Arm7),
      0x400_01a8..=0x400_01ae => {
        self.arm7_io_write_8(address, value as u8);
        self.arm7_io_write_8(address + 1, (value >> 8) as u8);
      }
      0x400_01b0..=0x400_01ba => println!("ignoring writes to key2 encryption seed io"),
      0x400_01c0 => self.write_spicnt(value),
      0x400_01c2 => self.write_spi_data(value as u8), // upper 8 bits are always ignored, even in bugged spi 16 bit mode. per the docs
      0x400_0204 => self.exmem.write(false, value),
      0x400_0206 => (),
      0x400_0208 => self.arm7.interrupt_master_enable = value & 0b1 != 0,
      0x400_0210 => {
        let mut val = self.arm7.interrupt_enable.bits() & 0xffff0000;

        val |= value as u32;


        self.arm7.interrupt_enable = InterruptEnableRegister::from_bits_retain(val);
      }
      0x400_0212 => {
        let mut val = self.arm7.interrupt_enable.bits() & 0xffff;

        val |= (value as u32) << 16;

        self.arm7.interrupt_enable = InterruptEnableRegister::from_bits_retain(val);
      }
      0x400_0214 => self.arm7.interrupt_request = InterruptRequestRegister::from_bits_retain(self.arm7.interrupt_request.bits() & !value as u32),
      0x400_0216 => self.arm7.interrupt_request = InterruptRequestRegister::from_bits_retain(self.arm7.interrupt_request.bits() & !((value as u32) << 16)),
      0x400_0300 => self.arm7.postflg |= value & 0b1 == 1,
      0x400_0304 => self.gpu.powcnt2 = PowerControlRegister2::from_bits_retain(value),
      0x400_0400..=0x400_04ff => self.arm7.apu.write_channels(address, value as u32, &mut self.scheduler, BitLength::Bit16),
      0x400_0500 => self.arm7.apu.soundcnt.write(value, None),
      0x400_0504 => self.arm7.apu.write_sound_bias(value, None),
      0x400_0508 => {
        self.arm7_io_write_8(address, value as u8);
        self.arm7_io_write_8(address + 1, (value >> 8) as u8);
      }
      0x400_0510 => self.arm7.apu.sndcapcnt[0].write_destination(value as u32, Some(0xff00)),
      0x400_0512 => self.arm7.apu.sndcapcnt[0].write_destination((value as u32) << 16, Some(0xff)),
      0x400_0514 => self.arm7.apu.sndcapcnt[0].write_length(value, None),
      0x400_0518 => self.arm7.apu.sndcapcnt[1].write_destination(value as u32, Some(0xff00)),
      0x400_051a => self.arm7.apu.sndcapcnt[1].write_destination((value as u32) << 16, Some(0xff)),
      0x400_051c => self.arm7.apu.sndcapcnt[1].write_length(value, None),
      0x400_1080 => (), // some kind of ds lite related register, can be safely ignored
      0x480_4000..=0x480_5fff => (),
      0x480_8000..=0x480_8fff => (),
      _ => {
        println!("[WARN] write to io register not implemented: {:X}", address)
      }
    }
  }

  pub fn arm7_io_write_8(&mut self, address: u32, value: u8) {
    // let address = if address & 0xffff == 0x8000 {
    //   0x400_0800
    // } else {
    //   address
    // };

    match address {
      0x400_0208 => self.arm7.interrupt_master_enable = value & 0b1 != 0,
      0x400_0138 => self.arm7.rtc.write(value as u16),
      0x400_01a0 => self.cartridge.spicnt.write(value as u16, self.exmem.nds_access_rights == AccessRights::Arm7, Some(0xff00)),
      0x400_01a1 => self.cartridge.spicnt.write((value as u16) << 8, self.exmem.nds_access_rights == AccessRights::Arm7, Some(0xff)),
      0x400_01a8..=0x400_01af => {
        let byte = address - 0x400_01a8;

        self.cartridge.write_command(value, byte as usize, self.exmem.nds_access_rights == AccessRights::Arm7);
      }
      0x400_01c2 => self.write_spi_data(value),
      0x400_0300 => self.arm7.postflg |= value & 0b1 == 1,
      0x400_0301 => self.write_haltcnt(value),
      0x400_0400..=0x400_04ff => self.arm7.apu.write_channels(address, value as u32, &mut self.scheduler, BitLength::Bit8),
      0x400_0500 => self.arm7.apu.soundcnt.write(value as u16, Some(0xff00)),
      0x400_0501 => self.arm7.apu.soundcnt.write((value as u16) << 8, Some(0xff)),
      0x400_0504 => self.arm7.apu.write_sound_bias(value as u16, Some(0xff00)),
      0x400_0505 => self.arm7.apu.write_sound_bias(((value & 0x3) as u16) << 8, Some(0xff)),
      0x400_0508 => self.arm7.apu.sndcapcnt[0].write(value),
      0x400_0509 => self.arm7.apu.sndcapcnt[1].write(value),
      _ => println!("[WARN] 8-bit write to unsupported io address: {:x}", address)
    }
  }
}