use crate::{cpu::registers::{interrupt_enable_register::InterruptEnableRegister, interrupt_request_register::InterruptRequestRegister}, gpu::registers::{bg_control_register::BgControlRegister, power_control_register1::PowerControlRegister1, window_in_register::WindowInRegister, window_out_register::WindowOutRegister}};

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
      0x400_00b0..=0x400_00ba => self.arm9.dma.read(0, (address - 0x400_00b0) as usize),
      0x400_00bc..=0x400_00c6 => self.arm9.dma.read(1, (address - 0x400_00bc) as usize),
      0x400_00c8..=0x400_00d2 => self.arm9.dma.read(2, (address - 0x400_00d2) as usize),
      0x400_00d4..=0x400_00de => self.arm9.dma.read(3, (address - 0x400_00d4) as usize),
      0x400_0208 => self.arm9.interrupt_master_enable as u32,
      0x400_0210 => self.arm9.interrupt_enable.bits(),
      0x400_0214 => self.arm9.interrupt_request.bits(),
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
      0x300_0000..=0x3ff_ffff => {
        if self.wramcnt.arm9_size == 0 {
          return 0;
        }

        let base = self.wramcnt.arm9_offset;

        let mask = self.wramcnt.arm9_size - 1;

        self.shared_wram[((address & mask) + base) as usize]
      }
      0x400_0000..=0x4ff_ffff => self.arm9_io_read_8(address),
      0x600_0000..=0x61f_ffff => self.gpu.vram.read_engine_a_bg(address),
      0x680_0000..=0x6ff_ffff => self.gpu.read_lcdc(address),
      // 0x700_0000..=0x7ff_ffff => 0,
      0x800_0000..=0x9ff_ffff => self.read_gba_rom(address, true),
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
      0x400_0006 => self.gpu.vcount,
      0x400_006c => self.gpu.engine_a.master_brightness.read(),
      0x400_00ba => self.arm9.dma.channels[0].dma_control.bits() as u16,
      0x400_0130 => self.key_input_register.bits(),
      0x400_0300 => self.arm9.postflg as u16,
      0x400_0180 => self.arm9.ipcsync.read() as u16,
      0x400_0182 => (self.arm9.ipcsync.read() >> 16) as u16,
      0x400_0184 => self.arm9.ipcfifocnt.read(&mut self.arm7.ipcfifocnt.fifo) as u16,
      0x400_0186 => (self.arm9.ipcfifocnt.read(&mut self.arm7.ipcfifocnt.fifo) >> 16) as u16,
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
      _ => {
        panic!("register not implemented: {:X}", address);
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
      0x300_0000..=0x3ff_ffff => {
        if self.wramcnt.arm9_size == 0 {
          return;
        }
        let arm9_offset = self.wramcnt.arm9_offset;
        let arm9_mask = self.wramcnt.arm9_size - 1;

        self.shared_wram[((address & arm9_mask) + arm9_offset) as usize] = val;
      }
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
      0x400_00b0..=0x400_00ba => self.arm9.dma.write(0, (address - 0x400_00b0) as usize, value, &mut self.scheduler),
      0x400_00bc..=0x400_00c6 => self.arm9.dma.write(1, (address - 0x400_00bc) as usize, value, &mut self.scheduler),
      0x400_00c8..=0x400_00d2 => self.arm9.dma.write(2, (address - 0x400_00d2) as usize, value, &mut self.scheduler),
      0x400_00d4..=0x400_00de => self.arm9.dma.write(3, (address - 0x400_00d4) as usize, value, &mut self.scheduler),
      0x400_00e0..=0x400_00ec => {
        let channel = (address - 0x400_00e0) / 4;

        self.arm9.dma_fill[channel as usize] = value;
      }
      0x400_0188 => self.send_to_fifo(true, value),
      0x400_0208 => self.arm9.interrupt_master_enable = value != 0,
      0x400_0210 => self.arm9.interrupt_enable = InterruptEnableRegister::from_bits_retain(value),
      0x400_0214 => {
        self.arm9.interrupt_request = InterruptRequestRegister::from_bits_retain(self.arm9.interrupt_request.bits() & !value);
      }
      0x400_0240 => {
        // need to write to 4 different registers
        for i in 0..4 {
          self.gpu.write_vramcnt(i, (value >> (8 * i)) as u8);
        }
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

        self.arm9.sqrt_result = self.start_sqrt_calculation();
      }
      0x400_02bc => {
        self.arm9.sqrt_param &= 0xffffffff;
        self.arm9.sqrt_param |= (value as u64) << 32;

        self.arm9.sqrt_result = self.start_sqrt_calculation();
      }
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

    let gpu = &mut self.gpu;

    macro_rules! write_bg_reference_point {
      (low $coordinate:ident $internal:ident $engine:ident $i:expr) => {{
        let existing = gpu.$engine.bg_props[$i].$coordinate as u32;

        let new_value = ((existing & 0xffff0000) + (value as u32)) as i32;

        gpu.$engine.bg_props[$i].$coordinate = new_value;
        gpu.$engine.bg_props[$i].$internal = new_value;
      }};
      (high $coordinate:ident $internal:ident $engine:ident $i:expr) => {{
        let existing = gpu.$engine.bg_props[$i].$coordinate;

        let new_value = existing & 0xffff | (((value & 0xfff) as i32) << 20) >> 4;

        gpu.$engine.bg_props[$i].$coordinate = new_value;
        gpu.$engine.bg_props[$i].$internal = new_value;
      }}
    }

    match address {
      0x400_0004 => self.gpu.dispstat[1].write(value),
      0x400_0006 => (),
      0x400_0008 => self.gpu.engine_a.bgcnt[0] = BgControlRegister::from_bits_retain(value),
      0x400_000a => self.gpu.engine_a.bgcnt[1] = BgControlRegister::from_bits_retain(value),
      0x400_000c => self.gpu.engine_a.bgcnt[2] = BgControlRegister::from_bits_retain(value),
      0x400_000e => self.gpu.engine_a.bgcnt[3] = BgControlRegister::from_bits_retain(value),
      0x400_0010 => self.gpu.engine_a.bgxofs[0] = value & 0b111111111,
      0x400_0012 => self.gpu.engine_a.bgyofs[0] = value & 0b111111111,
      0x400_0014 => self.gpu.engine_a.bgxofs[1] = value & 0b111111111,
      0x400_0016 => self.gpu.engine_a.bgyofs[1] = value & 0b111111111,
      0x400_0018 => self.gpu.engine_a.bgxofs[2] = value & 0b111111111,
      0x400_001a => self.gpu.engine_a.bgyofs[2] = value & 0b111111111,
      0x400_001c => self.gpu.engine_a.bgxofs[3] = value & 0b111111111,
      0x400_001e => self.gpu.engine_a.bgyofs[3] = value & 0b111111111,
      0x400_0020 => self.gpu.engine_a.bg_props[0].dx = value as i16,
      0x400_0022 => self.gpu.engine_a.bg_props[0].dmx = value as i16,
      0x400_0024 => self.gpu.engine_a.bg_props[0].dy = value as i16,
      0x400_0026 => self.gpu.engine_a.bg_props[0].dmy = value as i16,
      0x400_0028 => write_bg_reference_point!(low x internal_x engine_a 0),
      0x400_002a => write_bg_reference_point!(high x internal_x engine_a 0),
      0x400_002c => write_bg_reference_point!(low y internal_y engine_a 0),
      0x400_002e => write_bg_reference_point!(high y internal_y engine_a 0),
      0x400_0030 => self.gpu.engine_a.bg_props[1].dx = value as i16,
      0x400_0032 => self.gpu.engine_a.bg_props[1].dmx = value as i16,
      0x400_0034 => self.gpu.engine_a.bg_props[1].dy = value as i16,
      0x400_0036 => self.gpu.engine_a.bg_props[1].dmy = value as i16,
      0x400_0038 => write_bg_reference_point!(low x internal_x engine_a 1),
      0x400_003a => write_bg_reference_point!(high x internal_x engine_a 1),
      0x400_003c => write_bg_reference_point!(low y internal_y engine_a 1),
      0x400_003e => write_bg_reference_point!(high y internal_y engine_a 1),
      0x400_0040 => self.gpu.engine_a.winh[0].write(value),
      0x400_0042 => self.gpu.engine_a.winh[1].write(value),
      0x400_0044 => self.gpu.engine_a.winv[0].write(value),
      0x400_0046 => self.gpu.engine_a.winv[1].write(value),
      0x400_0048 => self.gpu.engine_a.winin = WindowInRegister::from_bits_retain(value),
      0x400_004a => self.gpu.engine_a.winout = WindowOutRegister::from_bits_retain(value),
      0x400_0050 => self.gpu.engine_a.bldcnt.write(value),
      0x400_0052 => self.gpu.engine_a.bldalpha.write(value),
      0x400_0054 => self.gpu.engine_a.bldy.write(value),
      0x400_006c => self.gpu.engine_a.master_brightness.write(value),
      0x400_00b0 => self.arm9.dma.channels[0].write_source(value as u32, 0xffff0000),
      0x400_00ba => self.arm9.dma.channels[0].write_control(value as u32, &mut self.scheduler),
      0x400_0180 => self.arm9.ipcsync.write(&mut self.arm7.ipcsync, &mut self.arm7.interrupt_request, value),
      0x400_0184 => {
        let previous_send_empty = self.arm9.ipcfifocnt.send_empty_irq;
        let previous_receive_not_empty = self.arm9.ipcfifocnt.receive_not_empty_irq;

        self.arm9.ipcfifocnt.write(&mut self.arm9.interrupt_request,&mut self.arm7.ipcfifocnt.fifo, value);

        // now check if there are any interrupts to send
        if self.arm7.ipcfifocnt.fifo.is_empty() && !previous_send_empty && self.arm7.ipcfifocnt.send_empty_irq {
          self.arm7.interrupt_request.insert(InterruptRequestRegister::IPC_SEND_FIFO_EMPTY);
        }
        if !self.arm9.ipcfifocnt.fifo.is_empty() && !previous_receive_not_empty && self.arm7.ipcfifocnt.send_empty_irq {
          self.arm7.interrupt_request.insert(InterruptRequestRegister::IPC_RECV_FIFO_NOT_EMPTY);
        }
      }
      0x400_01a0 => self.cartridge.spicnt.write(value as u32, 0xff00),
      0x400_01a2 => self.cartridge.spicnt.write((value as u32) << 16, 0xff),
      0x400_01a4 => self.cartridge.control.write(value as u32, 0xff00),
      0x400_01a6 => self.cartridge.control.write((value as u32) << 16, 0xff),
      0x400_0204 => self.exmem.write(true, value),
      0x400_0208 => self.arm9.interrupt_master_enable = value != 0,
      0x400_0240..=0x400_0246 => {
        let offset = address - 0x400_0240;

        self.gpu.write_vramcnt(offset, value as u8);

        if offset == 6 {
          // this is kinda hacky... so if we're writing to 400_0246 with a 16 bit value, the upper 8 bits actually
          // go to the wramcnt register instead of the next vramcnt register. TODO: fix this
          self.wramcnt.write((value >> 8) as u8);
        } else {
          self.gpu.write_vramcnt(offset + 1, (value >> 8) as u8);
        }
      }
      0x400_0248 => {
        let base = 7;

        // need to write to h and i vram registers
        for i in 0..2 {
          self.gpu.write_vramcnt(base + i, (value >> 8 * i) as u8);
        }
      }
      0x400_0280 => self.arm9.divcnt.write(value),
      0x400_02b0 => self.write_sqrtcnt(value),
      0x400_0304 => {
        let old_value = self.gpu.powcnt1.bits();

        let value = (old_value & 0xffff0000) | value as u32;

        self.gpu.powcnt1 = PowerControlRegister1::from_bits_retain(value);
      }
      _ => {
        panic!("register not implemented: {:X}", address)
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