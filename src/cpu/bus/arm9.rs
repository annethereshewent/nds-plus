use crate::{
  cpu::registers::{
    external_memory::AccessRights,
    interrupt_enable_register::InterruptEnableRegister,
    interrupt_request_register::InterruptRequestRegister
  },
  gpu::registers::{
    display_3d_control_register::Display3dControlRegister,
    power_control_register1::PowerControlRegister1
  }, number::Number
};

use super::{cp15::cp15_control_register::CP15ControlRegister, Bus, DTCM_SIZE, ITCM_SIZE, MAIN_MEMORY_SIZE};

impl Bus {
  pub fn arm9_mem_read_32(&mut self, address: u32) -> u32 {
    match address {
      0x400_0000..=0x4ff_ffff => self.arm9_io_read_32(address),
      _ => self.arm9_mem_read::<u32>(address)
    }
  }

  pub fn arm9_mem_read<T: Number>(&mut self, address: u32) -> T {
    let dtcm_ranges = self.arm9.cp15.dtcm_control.get_ranges();
    let itcm_ranges = self.arm9.cp15.itcm_control.get_ranges();

    if itcm_ranges.contains(&address) && !self.arm9.cp15.control.contains(CP15ControlRegister::ITCM_LOAD_MODE) {
      let actual_addr = (address + self.arm9.cp15.itcm_control.base_address()) & (ITCM_SIZE as u32 - 1);

      return unsafe { *(&self.itcm[actual_addr as usize] as *const u8 as *const T) };
    } else if dtcm_ranges.contains(&address) && !self.arm9.cp15.control.contains(CP15ControlRegister::DTCM_LOAD_MODE) {
      let actual_addr = (address + self.arm9.cp15.dtcm_control.base_address()) & (DTCM_SIZE as u32 - 1);

      return unsafe { *(&self.dtcm[actual_addr as usize] as *const u8 as *const T) }
    }

    if address >= 0xffff_0000 {
      return unsafe { *(&self.arm9.bios9[(address - 0xffff_0000) as usize] as *const u8 as *const T) };
    }

    match address {
      0x200_0000..=0x2ff_ffff => {
        return unsafe { *(&self.main_memory[(address & ((MAIN_MEMORY_SIZE as u32) - 1)) as usize] as *const u8 as *const T) }
      }
      0x300_0000..=0x3ff_ffff => {
        if self.wramcnt.arm9_size == 0 {
          return num::zero();
        }

        let base = self.wramcnt.arm9_offset;

        let mask = self.wramcnt.arm9_size - 1;

        return unsafe { *(&self.shared_wram[((address & mask) + base) as usize] as *const u8 as *const T) };
      }
      0x500_0000..=0x500_03ff => self.gpu.read_palette_a(address),
      0x500_0400..=0x500_07ff => self.gpu.read_palette_b(address),
      0x600_0000..=0x61f_ffff => self.gpu.vram.read_engine_a_bg(address),
      0x620_0000..=0x63f_ffff => self.gpu.vram.read_engine_b_bg(address),
      0x640_0000..=0x65f_ffff => self.gpu.vram.read_engine_a_obj(address),
      0x660_0000..=0x67f_ffff => self.gpu.vram.read_engine_b_obj(address),
      0x680_0000..=0x6ff_ffff => self.gpu.read_lcdc(address),
      0x700_0000..=0x7ff_ffff if address & 0x7ff < 0x400 => {
        unsafe { *(&self.gpu.engine_a.oam[(address & 0x3ff) as usize] as *const u8 as *const T) }
      }
      0x700_0000..=0x7ff_ffff => {
        unsafe { *(&self.gpu.engine_b.oam[(address & 0x3ff) as usize] as *const u8 as *const T) }
      }
      0x800_0000..=0x9ff_ffff => self.read_gba_rom(address, true),
      _ => {
        panic!("reading from unsupported address: {:X}", address);
      }
    }
  }

  pub fn arm9_io_read_32(&mut self, address: u32) -> u32 {
    match address {
      0x400_0000 => self.gpu.engine_a.dispcnt.read(),
      0x400_0004..=0x400_005f => self.arm9_io_read_16(address) as u32 | (self.arm9_io_read_16(address + 2) as u32) << 16,
      0x400_00b0..=0x400_00ba => self.arm9.dma.read(0, (address - 0x400_00b0) as usize),
      0x400_00bc..=0x400_00c6 => self.arm9.dma.read(1, (address - 0x400_00bc) as usize),
      0x400_00c8..=0x400_00d2 => self.arm9.dma.read(2, (address - 0x400_00c8) as usize),
      0x400_00d4..=0x400_00de => self.arm9.dma.read(3, (address - 0x400_00d4) as usize),
      0x400_00e0..=0x400_00ec => self.arm9.dma_fill[((address - 0x400_00e0)/4) as usize],
      0x400_01a4 => self.cartridge.control.read(self.exmem.nds_access_rights == AccessRights::Arm9),
      0x400_0208 => self.arm9.interrupt_master_enable as u32,
      0x400_0210 => self.arm9.interrupt_enable.bits(),
      0x400_0214 => self.arm9.interrupt_request.bits(),
      0x400_0240..=0x400_0246 | 0x400_0248..=0x400_0249 => self.arm9_io_read_16(address) as u32 | (self.arm9_io_read_16(address + 2) as u32) << 16,
      0x400_0280 => self.arm9.divcnt.read() as u32,
      0x400_0290 => self.arm9.div_numerator as u32,
      0x400_0294 => (self.arm9.div_numerator >> 32) as u32,
      0x400_0298 => self.arm9.div_denomenator as u32,
      0x400_029c => (self.arm9.div_denomenator >> 32) as u32,
      0x400_02a0 => self.arm9.div_result as u32,
      0x400_02a4 => (self.arm9.div_result >> 32) as u32,
      0x400_02a8 => self.arm9.div_remainder as u32,
      0x400_02ac => (self.arm9.div_remainder >> 32) as u32,
      0x400_02b4 => self.arm9.sqrt_result,
      0x400_02b8 => self.arm9.sqrt_param as u32,
      0x400_02bc => (self.arm9.sqrt_param >> 32) as u32,
      0x400_0440..=0x400_05c8 => 0,
      0x400_0600 => self.gpu.engine3d.read_geometry_status(&mut self.arm9.interrupt_request),
      0x400_0640..=0x400_067f => self.gpu.engine3d.read_clip_matrix(address),
      0x400_0680..=0x400_06a3 => self.gpu.engine3d.read_vector_matrix(address),
      0x400_1000 => self.gpu.engine_b.dispcnt.read(),
      0x400_1008..=0x400_105f => self.arm9_io_read_16(address) as u32 | (self.arm9_io_read_16(address + 2) as u32) << 16,
      0x400_4000..=0x400_4010 => 0, // DSi I/O ports
      0x410_0000 => self.receive_from_fifo(true),
      0x410_0010 => self.cartridge.read_gamecard_bus(&mut self.scheduler, self.exmem.nds_access_rights == AccessRights::Arm9, true),
      _ => {
        println!("[WARN] unsupported io address received: {:X}", address);
        0
      }
    }
  }

  pub fn arm9_mem_read_16(&mut self, address: u32) -> u16 {
    match address {
      0x400_0000..=0x4ff_ffff => self.arm9_io_read_16(address),
      _ => self.arm9_mem_read::<u16>(address)
    }
  }

  pub fn arm9_mem_read_8(&mut self, address: u32) -> u8 {
    match address {
      0x400_0000..=0x4ff_ffff => self.arm9_io_read_8(address),
      _ => self.arm9_mem_read::<u8>(address)
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
      0x400_0000 => self.gpu.engine_a.dispcnt.read() as u16,
      0x400_0002 => (self.gpu.engine_a.dispcnt.read() >> 16) as u16,
      0x400_0004 => self.gpu.dispstat[1].read(),
      0x400_0006 => self.gpu.vcount,
      0x400_0008..=0x400_005f => self.gpu.engine_a.read_register(address),
      0x400_0060 => self.gpu.engine3d.disp3dcnt.bits() as u16,
      0x400_006c => self.gpu.engine_a.master_brightness.read(),
      0x400_00b0..=0x400_00ba => {
        let actual_addr = address & !(0b11);
        let value = self.arm9.dma.read(0, (actual_addr - 0x400_00b0) as usize);

        if address & 0x3 != 2 {
          value as u16
        } else {
          (value >> 16) as u16
        }
      }
      0x400_00bc..=0x400_00c6 => {
        let actual_addr = address & !(0b11);
        let value = self.arm9.dma.read(1, (actual_addr - 0x400_00bc) as usize);

        if address & 0x3 != 2 {
          value as u16
        } else {
          (value >> 16) as u16
        }
      }
      0x400_00c8..=0x400_00d2 => {
        let actual_addr = address & !(0b11);
        let value = self.arm9.dma.read(2, (actual_addr - 0x400_00c8) as usize);

        if address & 0x3 != 2 {
          value as u16
        } else {
          (value >> 16) as u16
        }
      }
      0x400_00d4..=0x400_00de => {
        let actual_addr = address & !(0b11);
        let value = self.arm9.dma.read(3, (actual_addr - 0x400_00d4) as usize);

        if address & 0x3 != 2 {
          value as u16
        } else {
          (value >> 16) as u16
        }
      }
      0x400_00e0 => self.arm9.dma_fill[0] as u16,
      0x400_00e2 => (self.arm9.dma_fill[0] >> 16) as u16,
      0x400_00e4 => self.arm9.dma_fill[1] as u16,
      0x400_00e6 => (self.arm9.dma_fill[1] >> 16) as u16,
      0x400_00e8 => self.arm9.dma_fill[2] as u16,
      0x400_00ea => (self.arm9.dma_fill[2] >> 16) as u16,
      0x400_00ec => self.arm9.dma_fill[3] as u16,
      0x400_00ee => (self.arm9.dma_fill[3] >> 16) as u16,
      0x400_0100 => self.arm9.timers.t[0].read_timer_value(&self.scheduler),
      0x400_0102 => self.arm9.timers.t[0].timer_ctl.bits(),
      0x400_0104 => self.arm9.timers.t[1].read_timer_value(&self.scheduler),
      0x400_0106 => self.arm9.timers.t[1].timer_ctl.bits(),
      0x400_0108 => self.arm9.timers.t[2].read_timer_value(&self.scheduler),
      0x400_010a => self.arm9.timers.t[2].timer_ctl.bits(),
      0x400_010c => self.arm9.timers.t[3].read_timer_value(&self.scheduler),
      0x400_010e => self.arm9.timers.t[3].timer_ctl.bits(),
      0x400_0130 => self.key_input_register.bits(),
      0x400_0300 => self.arm9.postflg as u16,
      0x400_0180 => self.arm9.ipcsync.read() as u16,
      0x400_0182 => (self.arm9.ipcsync.read() >> 16) as u16,
      0x400_0184 => self.arm9.ipcfifocnt.read(&mut self.arm7.ipcfifocnt.fifo) as u16,
      0x400_0186 => (self.arm9.ipcfifocnt.read(&mut self.arm7.ipcfifocnt.fifo) >> 16) as u16,
      0x400_01a0 => self.cartridge.spicnt.read(self.exmem.nds_access_rights == AccessRights::Arm9),
      0x400_01a2 => self.cartridge.read_spidata(self.exmem.nds_access_rights == AccessRights::Arm9) as u16,
      0x400_01a4 => self.cartridge.control.read(self.exmem.nds_access_rights == AccessRights::Arm9) as u16,
      0x400_01a8 => self.cartridge.command[0] as u16 | (self.cartridge.command[1] as u16) << 8,
      0x400_01aa => self.cartridge.command[2] as u16 | (self.cartridge.command[3] as u16) << 8,
      0x400_01ac => self.cartridge.command[4] as u16 | (self.cartridge.command[5] as u16) << 8,
      0x400_01ae => self.cartridge.command[6] as u16 | (self.cartridge.command[7] as u16) << 8,
      0x400_0204 => self.exmem.read(true),
      0x400_0208 => self.arm9.interrupt_master_enable as u16,
      0x400_0240..=0x400_0246 => {
        let offset = address - 0x400_0240;

        let mut value = self.gpu.read_vramcnt(offset) as u16;
        value |= (self.gpu.read_vramcnt(offset + 1) as u16) << 8;

        value
      }
      0x400_0280 => self.arm9.divcnt.read(),
      0x400_0290 => self.arm9.div_numerator as u16,
      0x400_02b0 => self.arm9.sqrtcnt.read(),
      0x400_0304 => self.gpu.powcnt1.bits() as u16,
      0x400_0604 => self.gpu.engine3d.read_ram_count() as u16,
      0x400_0606 => (self.gpu.engine3d.read_ram_count() >> 16) as u16,
      0x400_0630..=0x400_0636 => 0, // unimplemented vectest
      0x400_1000 => self.gpu.engine_b.dispcnt.read() as u16,
      0x400_1002 => (self.gpu.engine_b.dispcnt.read() >> 16) as u16,
      0x400_1008..=0x400_105f => self.gpu.engine_b.read_register(address),
      0x400_106c => self.gpu.engine_b.master_brightness.read(),
      0x400_4000..=0x400_4fff => 0,
      _ => {
        println!("[WARN] read register not implemented: {:X}", address);
        0
      }
    }
  }

  fn arm9_io_read_8(&mut self, address: u32) -> u8 {
    match address {
      0x400_0240..=0x400_0246 => {
        let offset = address - 0x400_0240;

        self.gpu.read_vramcnt(offset)
      }
      0x400_0247 => self.wramcnt.read(),
      0x400_0248 => self.gpu.read_vramcnt(7),
      0x400_0249 => self.gpu.read_vramcnt(8),
      _ => {
        let val = self.arm9_io_read_16(address & !(0b1));

        if address & 0b1 == 1 {
          (val >> 8) as u8
        } else {
          (val & 0xff) as u8
        }
      }
    }
  }

  pub fn arm9_mem_write_32(&mut self, address: u32, val: u32) {
    match address {
      0x400_0000..=0x4ff_ffff => self.arm9_io_write_32(address, val),
      _ => self.arm9_mem_write::<u32>(address, val)
    }
  }

  pub fn arm9_mem_write_16(&mut self, address: u32, val: u16) {
    match address {
      0x400_0000..=0x4ff_ffff => self.arm9_io_write_16(address, val),
      _ => self.arm9_mem_write::<u16>(address, val)
    }
  }

  pub fn arm9_mem_write<T: Number>(&mut self, address: u32, val: T) {
    let dtcm_ranges = self.arm9.cp15.dtcm_control.get_ranges();
    let itcm_ranges = self.arm9.cp15.itcm_control.get_ranges();

    if itcm_ranges.contains(&address) {
      let actual_addr = (address + self.arm9.cp15.itcm_control.base_address()) & (ITCM_SIZE as u32 - 1);

      unsafe { *(&mut self.itcm[actual_addr as usize] as *mut u8 as *mut T) = val };

      return;
    }
    if dtcm_ranges.contains(&address) {
      let actual_addr = (address + self.arm9.cp15.dtcm_control.base_address()) & (DTCM_SIZE as u32 - 1);

      unsafe { *(&mut self.dtcm[actual_addr as usize] as *mut u8 as *mut T) = val };

      return;
    }

    match address {
      0x200_0000..=0x2ff_ffff => {
        unsafe { *(&mut self.main_memory[(address & ((MAIN_MEMORY_SIZE as u32) - 1)) as usize] as *mut u8 as *mut T) = val }
      }
      0x300_0000..=0x3ff_ffff => {
        if self.wramcnt.arm9_size == 0 {
          return;
        }
        let arm9_offset = self.wramcnt.arm9_offset;
        let arm9_mask = self.wramcnt.arm9_size - 1;

        unsafe { *(&mut self.shared_wram[((address & arm9_mask) + arm9_offset) as usize] as *mut u8 as *mut T) = val };
      }
      0x500_0000..=0x500_03ff => self.gpu.write_palette_a(address, val),
      0x500_0400..=0x500_07ff => self.gpu.write_palette_b(address, val),
      0x600_0000..=0x61f_ffff => self.gpu.vram.write_engine_a_bg(address, val),
      0x620_0000..=0x63f_ffff => self.gpu.vram.write_engine_b_bg(address, val),
      0x640_0000..=0x65f_ffff => self.gpu.vram.write_engine_a_obj(address, val),
      0x660_0000..=0x67f_ffff => self.gpu.vram.write_engine_b_obj(address, val),
      0x680_0000..=0x6ff_ffff => self.gpu.write_lcdc(address, val),
      0x700_0000..=0x7ff_ffff if address & 0x7ff < 0x400  => {
        unsafe { *(&mut self.gpu.engine_a.oam[(address & 0x3ff) as usize] as *mut u8 as *mut T) = val };
      }
      0x700_0000..=0x7ff_ffff => {
        unsafe { *(&mut self.gpu.engine_b.oam[(address & 0x3ff) as usize] as *mut u8 as *mut T) = val };
      }
      0x800_0000..=0x8ff_ffff => (),
      _ => {
        panic!("writing to unsupported address: {:X}", address);
      }
    }
  }

  pub fn arm9_mem_write_8(&mut self, address: u32, val: u8) {
    match address {
      0x400_0000..=0x4ff_ffff => self.arm9_io_write_8(address, val),
      _ => self.arm9_mem_write::<u8>(address, val)
    }
  }

  pub fn arm9_io_write_32(&mut self, address: u32, value: u32) {
    match address {
      0x400_0000 => self.gpu.engine_a.dispcnt.write(value, None),
      0x400_0004 => {
        self.arm9_io_write_16(address, value as u16);
        self.arm9_io_write_16(address + 2, (value >> 16) as u16);
      }
      0x400_0008..=0x400_005f => {
        self.arm9_io_write_16(address, value as u16);
        self.arm9_io_write_16(address + 2, (value >> 16) as u16);
      }
      0x400_0060 => self.gpu.engine3d.disp3dcnt = Display3dControlRegister::from_bits_retain(value),
      0x400_0064 => self.gpu.dispcapcnt.write(value),
      0x400_0068 => (), // ignoring writes to DISP_MMEM_FIFO
      0x400_006c => self.gpu.engine_a.master_brightness.write(value as u16),
      0x400_00b0..=0x400_00ba => self.arm9.dma.write(0, (address - 0x400_00b0) as usize, value, None, &mut self.scheduler),
      0x400_00bc..=0x400_00c6 => self.arm9.dma.write(1, (address - 0x400_00bc) as usize, value, None, &mut self.scheduler),
      0x400_00c8..=0x400_00d2 => self.arm9.dma.write(2, (address - 0x400_00c8) as usize, value, None, &mut self.scheduler),
      0x400_00d4..=0x400_00de => self.arm9.dma.write(3, (address - 0x400_00d4) as usize, value, None, &mut self.scheduler),
      0x400_00e0..=0x400_00ec => {
        let channel = (address - 0x400_00e0) / 4;

        self.arm9.dma_fill[channel as usize] = value;
      }
      0x400_01a0 => {
        self.arm9_io_write_16(address, value as u16);
        self.arm9_io_write_16(address + 2, (value >> 16) as u16);
      }
      0x400_01a4 => self.cartridge.write_control(value, None, &mut self.scheduler, true, self.exmem.nds_access_rights == AccessRights::Arm9),
      0x400_01a8..=0x400_01ac => {
        self.arm9_io_write_16(address, value as u16);
        self.arm9_io_write_16(address + 2, (value >> 16) as u16);
      }
      0x400_0188 => self.send_to_fifo(true, value),
      0x400_0208 => self.arm9.interrupt_master_enable = value & 0b1 != 0,
      0x400_0210 => self.arm9.interrupt_enable = InterruptEnableRegister::from_bits_retain(value),
      0x400_0214 => {
        self.arm9.interrupt_request = InterruptRequestRegister::from_bits_retain(self.arm9.interrupt_request.bits() & !value);

        self.gpu.engine3d.check_interrupts(&mut self.arm9.interrupt_request);
      }
      0x400_0240..=0x400_0249 => {
        self.arm9_io_write_16(address, value as u16);
        self.arm9_io_write_16(address + 2, (value >> 16) as u16);
      }
      0x400_0290 => {
        self.arm9.div_numerator &= 0xffffffff00000000;
        self.arm9.div_numerator |= value as u64;

        self.start_div_calculation();
      }
      0x400_0294 => {
        self.arm9.div_numerator &= 0xffffffff;
        self.arm9.div_numerator |= (value as u64) << 32;

        self.start_div_calculation();
      }
      0x400_0298 => {
        self.arm9.div_denomenator &= 0xffffffff00000000;
        self.arm9.div_denomenator |= value as u64;

        self.start_div_calculation();
      }
      0x400_029c => {
        self.arm9.div_denomenator &= 0xffffffff;
        self.arm9.div_denomenator |= (value as u64) << 32;

        self.start_div_calculation();
      }
      0x400_02b8 => {
        self.arm9.sqrt_param &= 0xffffffff00000000;
        self.arm9.sqrt_param |= value as u64;

        self.start_sqrt_calculation();
      }
      0x400_02bc => {
        self.arm9.sqrt_param &= 0xffffffff;
        self.arm9.sqrt_param |= (value as u64) << 32;

        self.start_sqrt_calculation();
      }
      0x400_0304 => self.gpu.powcnt1 = PowerControlRegister1::from_bits_retain(value as u16),
      0x400_0330..=0x400_033f => {
        self.arm9_io_write_16(address, value as u16);
        self.arm9_io_write_16(address + 2, (value >> 16) as u16);
      }
      0x400_0350 => self.gpu.engine3d.write_clear_color(value),
      0x400_0358 => self.gpu.engine3d.write_fog_color(value),
      0x400_0360..=0x400_03bf => {
        // toon table and fog table
        self.arm9_io_write_16(address, value as u16);
        self.arm9_io_write_16(address + 2, (value >> 16) as u16);
      }
      0x400_0400..=0x400_043f => self.gpu.engine3d.write_geometry_fifo(value, &mut self.arm9.interrupt_request),
      0x400_0440..=0x400_05c8 => {
        self.gpu.engine3d.write_geometry_command(address, value, &mut self.arm9.interrupt_request);
        if self.gpu.engine3d.should_run_dmas() {
          self.arm9.dma.notify_geometry_fifo_event();
          self.arm7.dma.notify_geometry_fifo_event();
        }
      }
      0x400_0600 => self.gpu.engine3d.write_geometry_status(value, &mut self.arm9.interrupt_request, None),
      0x400_1000 => self.gpu.engine_b.dispcnt.write(value, None),
      0x400_1004 => (),
      0x400_1008..=0x400_105f => {
        self.arm9_io_write_16(address, value as u16);
        self.arm9_io_write_16(address + 2, (value >> 16) as u16);
      }
      0x400_1060..=0x400_1068 => (),
      0x400_4000..=0x400_4fff => (),
      0x400_106c => self.gpu.engine_b.master_brightness.write(value as u16),
      _ => println!("[WARN] write to unsupported io address: {:X}", address)
    }
  }

  pub fn arm9_io_write_16(&mut self, address: u32, value: u16) {
    match address {
      0x400_0000 => self.gpu.engine_a.dispcnt.write(value as u32, Some(0xffff0000)),
      0x400_0002 => self.gpu.engine_a.dispcnt.write((value as u32) << 16, Some(0xffff)),
      0x400_0004 => self.gpu.dispstat[1].write(value),
      0x400_0006 => (),
      0x400_0008..=0x400_005f => self.gpu.engine_a.write_register(address, value, None),
      0x400_0060 => self.gpu.engine3d.disp3dcnt = Display3dControlRegister::from_bits_retain(value as u32),
      0x400_006c => self.gpu.engine_a.master_brightness.write(value),
      0x400_00b0..=0x400_00ba => {
        let actual_addr = address & !(0x3);

        if address & 0x3 != 2 {
          self.arm9.dma.write(0, (actual_addr - 0x400_00b0) as usize,  value as u32, Some(0xffff0000), &mut self.scheduler);
        } else {
          self.arm9.dma.write(0, (actual_addr - 0x400_00b0) as usize,  (value as u32) << 16, Some(0xffff), &mut self.scheduler);
        }
      }
      0x400_00bc..=0x400_00c6 => {
        let actual_addr = address & !(0x3);

        if address & 0x3 != 2 {
          self.arm9.dma.write(1, (actual_addr - 0x400_00bc) as usize, value as u32, Some(0xffff0000), &mut self.scheduler);
        } else {
          self.arm9.dma.write(1, (actual_addr - 0x400_00bc) as usize, (value as u32) << 16, Some(0xffff), &mut self.scheduler);
        }
      }
      0x400_00c8..=0x400_00d2 => {
        let actual_addr = address & !(0x3);

        if address & 0x3 != 2 {
          self.arm9.dma.write(2, (actual_addr - 0x400_00c8) as usize, value as u32, Some(0xffff0000), &mut self.scheduler);
        } else {
          self.arm9.dma.write(2, (actual_addr - 0x400_00c8) as usize, (value as u32) << 16, Some(0xffff), &mut self.scheduler);
        }
      }
      0x400_00d4..=0x400_00de => {
        let actual_addr = address & !(0x3);

        if address & 0x3 != 2 {
          self.arm9.dma.write(3, (actual_addr - 0x400_00d4) as usize, value as u32, Some(0xffff0000), &mut self.scheduler);
        } else {
          self.arm9.dma.write(3, (actual_addr - 0x400_00d4) as usize, (value as u32) << 16, Some(0xffff), &mut self.scheduler);
        }
      }
      0x400_0100 => self.arm9.timers.t[0].reload_timer_value(value),
      0x400_0102 => self.arm9.timers.t[0].write_timer_control(value, &mut self.scheduler),
      0x400_0104 => self.arm9.timers.t[1].reload_timer_value(value),
      0x400_0106 => self.arm9.timers.t[1].write_timer_control(value, &mut self.scheduler),
      0x400_0108 => self.arm9.timers.t[2].reload_timer_value(value),
      0x400_010a => self.arm9.timers.t[2].write_timer_control(value, &mut self.scheduler),
      0x400_010c => self.arm9.timers.t[3].reload_timer_value(value),
      0x400_010e => self.arm9.timers.t[3].write_timer_control(value, &mut self.scheduler),
      0x400_0180 => self.arm9.ipcsync.write(&mut self.arm7.ipcsync, &mut self.arm7.interrupt_request, value),
      0x400_0184 => {
        self.arm9.ipcfifocnt.write(&mut self.arm9.interrupt_request,&mut self.arm7.ipcfifocnt.fifo, value);
      }
      0x400_01a0 => self.cartridge.spicnt.write(value, self.exmem.nds_access_rights == AccessRights::Arm9, None),
      0x400_01a2 => self.cartridge.write_spidata(value as u8, self.exmem.nds_access_rights == AccessRights::Arm9),
      0x400_01a4 => self.cartridge.write_control(value as u32, Some(0xffff0000), &mut self.scheduler, true, self.exmem.nds_access_rights == AccessRights::Arm9),
      0x400_01a6 => self.cartridge.write_control((value as u32) << 16, Some(0xffff), &mut self.scheduler, true, self.exmem.nds_access_rights == AccessRights::Arm9),
      0x400_01a8..=0x400_01ae => {
        self.arm9_io_write_8(address, value as u8);
        self.arm9_io_write_8(address + 1, (value >> 8) as u8);
      }
      0x400_0204 => self.exmem.write(true, value),
      0x400_0208 => self.arm9.interrupt_master_enable = value & 0b1 != 0,
      0x400_0240..=0x400_0249 => {
        self.arm9_io_write_8(address, value as u8);
        self.arm9_io_write_8(address + 1, (value >> 8) as u8);
      }
      0x400_0280 => self.arm9.divcnt.write(value),
      0x400_02b0 => self.write_sqrtcnt(value),
      0x400_0300 => self.arm9.postflg |= value & 0b1 == 1,
      0x400_0304 => self.gpu.powcnt1 = PowerControlRegister1::from_bits_retain(value),
      0x400_0330..=0x400_033f => self.gpu.engine3d.write_edge_color(address, value),
      0x400_0340 => self.gpu.engine3d.write_alpha_ref(value),
      0x400_0354 => self.gpu.engine3d.write_clear_depth(value),
      0x400_0356 => self.gpu.engine3d.write_clear_image_offset(value),
      0x400_035c => self.gpu.engine3d.write_fog_offset(value),
      0x400_0360..=0x400_037f => {
        self.arm9_io_write_8(address, value as u8);
        self.arm9_io_write_8(address + 1, (value >> 8) as u8);
      }
      0x400_0380..=0x400_03bf => self.gpu.engine3d.write_toon_table(address, value),
      0x400_0610 => (),
      0x400_1000 => self.gpu.engine_b.dispcnt.write(value as u32, Some(0xffff0000)),
      0x400_1002 => self.gpu.engine_b.dispcnt.write((value as u32) << 16, Some(0xffff)),
      0x400_1008..=0x400_105f => self.gpu.engine_b.write_register(address, value, None),
      0x400_106c => self.gpu.engine_b.master_brightness.write(value),
      _ => {
        println!("[WARN] write to register not implemented: {:X}", address)
      }
    }
  }

  pub fn arm9_io_write_8(&mut self, address: u32, value: u8) {
    match address {
      0x400_004c => self.gpu.mosaic.write(value as u16, 0xff00),
      0x400_004d => self.gpu.mosaic.write((value as u16) << 8, 0xff),
      0x400_01a1 => self.cartridge.spicnt.write((value as u16) << 8, self.exmem.nds_access_rights == AccessRights::Arm9, Some(0xff)),
      0x400_01a8..=0x400_01af => {
        let byte = address - 0x400_01a8;

        self.cartridge.write_command(value, byte as usize, self.exmem.nds_access_rights == AccessRights::Arm9);
      }
      0x400_0208 => self.arm9.interrupt_master_enable = value & 0b1 != 0,
      0x400_0240..=0x400_0246 => {
        let offset = address - 0x400_0240;

        self.gpu.write_vramcnt(offset, value);
      }
      0x400_0247 => self.wramcnt.write(value),
      0x400_0248 => self.gpu.write_vramcnt(7, value),
      0x400_0249 => self.gpu.write_vramcnt(8, value),
      0x400_0360..=0x400_037f => self.gpu.engine3d.write_fog_table(address, value),
      0x400_0600 => self.gpu.engine3d.write_geometry_status(value as u32, &mut self.arm9.interrupt_request, Some(0xffffff00)),
      0x400_0601 => self.gpu.engine3d.write_geometry_status((value as u32) << 8, &mut self.arm9.interrupt_request, Some(0xffff00ff)),
      0x400_0602 => self.gpu.engine3d.write_geometry_status((value as u32) << 16, &mut self.arm9.interrupt_request, Some(0xff00ffff)),
      0x400_0603 => self.gpu.engine3d.write_geometry_status((value as u32) << 24, &mut self.arm9.interrupt_request, Some(0x00ffffff)),
      0x400_1008..=0x400_105f => {
        let actual_address = address & !(0b1);

        if address & 0b1 == 0 {
          self.gpu.engine_b.write_register(actual_address, value as u16, Some(0xff00));
        } else {
          self.gpu.engine_b.write_register(actual_address, (value as u16) << 8, Some(0xff));
        }
      }
      _ => println!("[WARN] 8-bit write to unsupported io address {:x}", address)
    }
  }
}
