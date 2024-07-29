use std::{thread::sleep, time::{Duration, SystemTime, UNIX_EPOCH}};

use engine_2d::Engine2d;
use engine_3d::Engine3d;
use registers::{
  display_3d_control_register::Display3dControlRegister,
  display_capture_control_register::{
    CaptureSource,
    DisplayCaptureControlRegister,
    ScreenSourceA, ScreenSourceB
  },
  display_control_register::{
    BgMode,
    DisplayMode
  },
  display_status_register::{
    DispStatFlags,
    DisplayStatusRegister
  },
  power_control_register1::PowerControlRegister1,
  power_control_register2::PowerControlRegister2,
  vram_control_register::VramControlRegister
};
use vram::{Bank, VRam};

use crate::{
  cpu::{
    dma::{
      dma_channel::registers::dma_control_register::DmaTiming,
      dma_channels::DmaChannels
    },
    registers::{
      interrupt_request_register::InterruptRequestRegister,
      mosaic_register::MosaicRegister
    }
  },
  scheduler::{
    EventType,
    Scheduler
  }
};

pub mod registers;
pub mod engine_2d;
pub mod engine_3d;
pub mod vram;
pub mod color;

const NUM_LINES: u16 = 263;

pub const SCREEN_HEIGHT: u16 = 192;
pub const SCREEN_WIDTH: u16 = 256;

pub const HBLANK_CYCLES: usize = 1606;
pub const HDRAW_CYCLES: usize = 524;

pub const FPS_INTERVAL: u128 = 1000 / 60;

const BANK_A: u32 = Bank::BankA as u32;
const BANK_B: u32 = Bank::BankB as u32;
const BANK_C: u32 = Bank::BankC as u32;
const BANK_D: u32 = Bank::BankD as u32;
const BANK_E: u32 = Bank::BankE as u32;
const BANK_F: u32 = Bank::BankF as u32;
const BANK_G: u32 = Bank::BankG as u32;
const BANK_H: u32 = Bank::BankH as u32;
const BANK_I: u32 = Bank::BankI as u32;

#[derive(Copy, Clone)]
pub struct BgProps {
  pub x: i32,
  pub y: i32,
  pub dx: i16,
  pub dmx: i16,
  pub dy: i16,
  pub dmy: i16,
  pub internal_x: i32,
  pub internal_y: i32
}

impl BgProps {
  pub fn new() -> Self {
    Self {
      x: 0,
      y: 0,
      dx: 0,
      dmx: 0,
      dy: 0,
      dmy: 0,
      internal_x: 0,
      internal_y: 0
    }
  }
}

pub struct GPU {
  pub engine_a: Engine2d<false>,
  pub engine_b: Engine2d<true>,
  pub engine3d: Engine3d,
  pub powcnt1: PowerControlRegister1,
  pub powcnt2: PowerControlRegister2,
  pub vramcnt: [VramControlRegister; 9],
  pub dispstat: [DisplayStatusRegister; 2],
  pub frame_finished: bool,
  pub vram: VRam,
  pub vcount: u16,
  pub dispcapcnt: DisplayCaptureControlRegister,
  pub mosaic: MosaicRegister,
  pub disp3dcnt: Display3dControlRegister,
  pub is_capturing: bool,
  previous_time: u128
}

impl GPU {
  pub fn new(scheduler: &mut Scheduler) -> Self {
    let mut vramcnt: Vec<VramControlRegister> = Vec::new();

    for i in 0..9 {
      vramcnt.push(VramControlRegister::new(i));
    }

    let gpu = Self {
      engine_a: Engine2d::new(),
      engine_b: Engine2d::new(),
      engine3d: Engine3d::new(),
      powcnt1: PowerControlRegister1::from_bits_retain(0),
      powcnt2: PowerControlRegister2::from_bits_retain(0),
      vramcnt: vramcnt.try_into().unwrap(),
      dispstat: [DisplayStatusRegister::new(), DisplayStatusRegister::new()],
      dispcapcnt: DisplayCaptureControlRegister::new(),
      vcount: 0,
      frame_finished: false,
      vram: VRam::new(),
      mosaic: MosaicRegister::new(),
      disp3dcnt: Display3dControlRegister::from_bits_retain(0),
      is_capturing: false,
      previous_time: 0
    };

    scheduler.schedule(EventType::HBlank, HBLANK_CYCLES);

    gpu
  }

  pub fn cap_fps(&mut self) {
    let current_time = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("an error occurred")
      .as_millis();

    if self.previous_time != 0 {
      let diff = current_time - self.previous_time;

      if diff < FPS_INTERVAL {
        sleep(Duration::from_millis((FPS_INTERVAL - diff) as u64));
      }
    }

    self.previous_time = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("an error occurred")
      .as_millis();
  }

  pub fn handle_hblank(
    &mut self,
    scheduler: &mut Scheduler,
    interrupt_requests: &mut [&mut InterruptRequestRegister],
    dma_channels: &mut [&mut DmaChannels],
    cycles_left: usize)
  {
    self.schedule_hdraw(scheduler, cycles_left);

    for dispstat in &mut self.dispstat {
      dispstat.flags.insert(DispStatFlags::HBLANK);
    }

    for dma in dma_channels {
      dma.notify_gpu_event(DmaTiming::Hblank);
    }

    if self.vcount < SCREEN_HEIGHT {
      self.render_line();
    }

    self.check_interrupts(DispStatFlags::HBLANK_IRQ_ENABLE, InterruptRequestRegister::HBLANK, interrupt_requests);
  }

  pub fn check_interrupts(&mut self, dispstat_flag: DispStatFlags, interrupt_flag: InterruptRequestRegister, interrupt_requests: &mut [&mut InterruptRequestRegister]) {
    for i in 0..2 {
      let dispstat = &mut self.dispstat[i];
      let interrupt_request = &mut interrupt_requests[i];

      if dispstat.flags.contains(dispstat_flag) {
        interrupt_request.insert(interrupt_flag);
      }
    }
  }

  pub fn start_next_line(
    &mut self, scheduler: &mut Scheduler,
    interrupt_requests: &mut [&mut InterruptRequestRegister],
    dma_channels: &mut [&mut DmaChannels],
    cycles_left: usize)
  {
    scheduler.schedule(EventType::HBlank, HBLANK_CYCLES - cycles_left);

    self.engine_a.clear_obj_lines();
    self.engine_b.clear_obj_lines();

    self.vcount += 1;

    if self.vcount == NUM_LINES {
      self.vcount = 0;

      self.engine_a.on_end_vblank();
      self.engine_b.on_end_vblank();
    }

    if self.vcount == 0 {
      self.is_capturing = self.dispcapcnt.capture_enable;

      for dispstat in &mut self.dispstat {
        dispstat.flags.remove(DispStatFlags::VBLANK);
      }
    } else if self.vcount == SCREEN_HEIGHT {
      if self.is_capturing {
        self.dispcapcnt.capture_enable = false;
      }
      self.trigger_vblank();

      for dma in dma_channels {
        dma.notify_gpu_event(DmaTiming::Vblank);
      }

      self.frame_finished = true;

      self.check_interrupts(DispStatFlags::VBLANK_IRQ_ENABLE, InterruptRequestRegister::VBLANK, interrupt_requests);
    } else if self.vcount == NUM_LINES - 48 {
      // per martin korth, "Rendering starts 48 lines in advance (while still in the Vblank period)"
      self.engine3d.clear_frame_buffer();
      self.engine3d.start_rendering(&self.vram);

      self.engine3d.execute_commands(&mut interrupt_requests[1]);

      if self.engine3d.should_run_dmas() {
        for dma in dma_channels {
          dma.notify_geometry_fifo_event();
        }
      }
    }

    for i in 0..2 {
      let dispstat = &mut self.dispstat[i];
      let interrupt_request = &mut interrupt_requests[i];

      if dispstat.flags.contains(DispStatFlags::VCOUNTER_IRQ_ENABLE) && self.vcount == dispstat.vcount_setting {
        interrupt_request.insert(InterruptRequestRegister::VCOUNTER_MATCH);
      }
    }
  }

  fn start_capture_image(&mut self) {
    let width = self.dispcapcnt.get_capture_width() as usize;
    let start_address = self.vcount as usize * SCREEN_WIDTH as usize;
    let block = self.engine_a.dispcnt.vram_block;

    if self.dispcapcnt.source_a == ScreenSourceA::Screen3d || self.engine_a.dispcnt.bg_mode != BgMode::Mode0 {
      todo!("3d not implemented yet");
    }

    if self.dispcapcnt.source_b == ScreenSourceB::MainMemoryDisplayFifo {
      todo!("main memory display fifo not implemented");
    }

    let read_offset = if self.engine_a.dispcnt.display_mode != DisplayMode::Mode2 {
      2 * start_address + self.dispcapcnt.vram_read_offset as usize
    } else {
      2 * start_address
    };

    let mut source_b: [u8; 2 * SCREEN_WIDTH as usize] = [0; 2 * SCREEN_WIDTH as usize];

    source_b[..2 * width].copy_from_slice(&self.vram.banks[block as usize][read_offset..read_offset + 2 * width]);

    let write_offset = 2 * start_address as usize + self.dispcapcnt.vram_write_offset as usize;
    let write_block = self.dispcapcnt.vram_write_block as usize;

    fn process_channels(channel_a: u16, channel_b: u16, a_alpha: u16, b_alpha: u16, eva: u16, evb: u16) -> u8 {
      /*
        Dest_Intensity = (  (SrcA_Intensitity * SrcA_Alpha * EVA)
          + (SrcB_Intensitity * SrcB_Alpha * EVB) ) / 16
        */
      ((channel_a * a_alpha * eva + channel_b * b_alpha * evb) / 16) as u8
    }

    // finally transfer the capture image!
    match self.dispcapcnt.capture_source {
      CaptureSource::SourceA => {
        let mut index = 0;
        for address in start_address..start_address+width {
          let r = self.engine_a.pixels[3 * address] >> 3;
          let g = self.engine_a.pixels[3 * address + 1] >> 3;
          let b = self.engine_a.pixels[3 * address + 2] >> 3;

          let pixel = (r as u16) & 0x1f | (g as u16) & 0x1f << 5 | (b as u16) & 0x1f << 10;

          self.vram.banks[write_block][write_offset + 2 * index] = pixel as u8;
          self.vram.banks[write_block][write_offset + 2 * index + 1] = (pixel >> 8) as u8;

          index += 1;
        }
      }
      CaptureSource::SourceB => {
        self.vram.banks[write_block][write_offset..write_offset + 2 * width].copy_from_slice(&source_b[..2 * width]);
      }
      CaptureSource::Blended => {
        let mut index: usize = 0;
        for address_a in start_address..start_address+width {
          let r_a = self.engine_a.pixels[3 * address_a] >> 3;
          let g_a = self.engine_a.pixels[3 * address_a + 1] >> 3;
          let b_a = self.engine_a.pixels[3 * address_a + 2] >> 3;

          let pixel_b = source_b[index] as u16 | (source_b[index] as u16) << 8;

          // TODO: colors are converted from rgb15 to rgb24 and lose the alpha bit. need to find
          // a way around that
          let alpha_a = 1 as u8;
          let alpha_b = (pixel_b >> 15 & 0b1) as u8;

          let r_b = (pixel_b & 0x1f) as u8;
          let g_b = ((pixel_b >> 5) & 0x1f) as u8;
          let b_b = ((pixel_b >> 10) & 0x1f) as u8;


          let new_r = process_channels(
            r_a as u16,
            r_b as u16,
            alpha_a as u16,
            alpha_b as u16,
            self.dispcapcnt.eva as u16,
            self.dispcapcnt.evb as u16
          );
          let new_g = process_channels(
            g_a as u16,
            g_b as u16,
            alpha_a as u16,
            alpha_b as u16,
            self.dispcapcnt.eva as u16,
            self.dispcapcnt.evb as u16
          );
          let new_b = process_channels(
            b_a as u16,
            b_b as u16,
            alpha_a as u16,
            alpha_b as u16,
            self.dispcapcnt.eva as u16,
            self.dispcapcnt.evb as u16
          );
          // Dest_Alpha = (SrcA_Alpha AND (EVA>0)) OR (SrcB_Alpha AND EVB>0))
          let alpha = (alpha_a > 0 && self.dispcapcnt.eva > 0) || (alpha_b > 0 && self.dispcapcnt.evb > 0);

          let new_color = (new_r as u16) & 0x1f | ((new_g as u16) & 0x1f) << 5 | ((new_b as u16) & 0x1f) << 10 | (alpha as u16) << 15;

          self.vram.banks[write_block][write_offset + 2 * index] = new_color as u8;
          self.vram.banks[write_block][write_offset + 2 * index + 1] = (new_color >> 8) as u8;

          index += 1;
        }
      }
    }
  }

  pub fn write_palette_a(&mut self, address: u32, val: u8) {
    self.engine_a.write_palette_ram(address, val);
  }

  pub fn read_palette_a(&self, address: u32) -> u8 {
    self.engine_a.read_palette_ram(address)
  }

  pub fn read_palette_b(&self, address: u32) -> u8 {
    self.engine_b.read_palette_ram(address)
  }

  pub fn write_palette_b(&mut self, address: u32, val: u8) {
    self.engine_b.write_palette_ram(address, val);
  }

  pub fn write_lcdc(&mut self, address: u32, val: u8) {
    match address {
      0x680_0000..=0x681_ffff => self.vram.write_lcdc_bank(Bank::BankA, address, val),
      0x682_0000..=0x683_ffff => self.vram.write_lcdc_bank(Bank::BankB, address, val),
      0x684_0000..=0x685_ffff => self.vram.write_lcdc_bank(Bank::BankC, address, val),
      0x686_0000..=0x687_ffff => self.vram.write_lcdc_bank(Bank::BankD, address, val),
      0x688_0000..=0x688_ffff => self.vram.write_lcdc_bank(Bank::BankE, address, val),
      0x689_0000..=0x689_3fff => self.vram.write_lcdc_bank(Bank::BankF, address, val),
      0x689_4000..=0x689_7fff => self.vram.write_lcdc_bank(Bank::BankG, address, val),
      0x689_8000..=0x689_ffff => self.vram.write_lcdc_bank(Bank::BankH, address, val),
      0x68a_0000..=0x68a_3fff => self.vram.write_lcdc_bank(Bank::BankI, address, val),
      _ => unreachable!("received address: {:X}", address)
    }
  }

  pub fn read_lcdc(&mut self, address: u32) -> u8 {
    match address {
      0x680_0000..=0x681_ffff => self.vram.read_lcdc_bank(Bank::BankA, address),
      0x682_0000..=0x683_ffff => self.vram.read_lcdc_bank(Bank::BankB, address),
      0x684_0000..=0x685_ffff => self.vram.read_lcdc_bank(Bank::BankC, address),
      0x686_0000..=0x687_ffff => self.vram.read_lcdc_bank(Bank::BankD, address),
      0x688_0000..=0x688_ffff => self.vram.read_lcdc_bank(Bank::BankE, address),
      0x689_0000..=0x689_3fff => self.vram.read_lcdc_bank(Bank::BankF, address),
      0x689_4000..=0x689_7fff => self.vram.read_lcdc_bank(Bank::BankG, address),
      0x689_8000..=0x689_ffff => self.vram.read_lcdc_bank(Bank::BankH, address),
      0x68a_0000..=0x68a_3fff => self.vram.read_lcdc_bank(Bank::BankI, address),
      _ => unreachable!("received address: {:X}", address)
    }
  }

  pub fn read_arm7_wram(&self, address: u32) -> u8 {
    self.vram.read_arm7_wram(address)
  }

  pub fn write_vramcnt(&mut self, offset: u32, val: u8) {
    if self.vramcnt[offset as usize].vram_enable {
      match offset {
        BANK_A => self.vram.unmap_bank(Bank::BankA, &self.vramcnt[offset as usize]),
        BANK_B => self.vram.unmap_bank(Bank::BankB, &self.vramcnt[offset as usize]),
        BANK_C => self.vram.unmap_bank(Bank::BankC, &self.vramcnt[offset as usize]),
        BANK_D => self.vram.unmap_bank(Bank::BankD, &self.vramcnt[offset as usize]),
        BANK_E => self.vram.unmap_bank(Bank::BankE, &self.vramcnt[offset as usize]),
        BANK_F => self.vram.unmap_bank(Bank::BankF, &self.vramcnt[offset as usize]),
        BANK_G => self.vram.unmap_bank(Bank::BankG, &self.vramcnt[offset as usize]),
        BANK_H => self.vram.unmap_bank(Bank::BankH, &self.vramcnt[offset as usize]),
        BANK_I => self.vram.unmap_bank(Bank::BankI, &self.vramcnt[offset as usize]),
        _ => unreachable!("can't happen")
      }
    }

    self.vramcnt[offset as usize].write(val);

    if self.vramcnt[offset as usize].vram_enable {
      match offset {
        BANK_A => self.vram.map_bank(Bank::BankA, &self.vramcnt[offset as usize]),
        BANK_B => self.vram.map_bank(Bank::BankB, &self.vramcnt[offset as usize]),
        BANK_C => self.vram.map_bank(Bank::BankC, &self.vramcnt[offset as usize]),
        BANK_D => self.vram.map_bank(Bank::BankD, &self.vramcnt[offset as usize]),
        BANK_E => self.vram.map_bank(Bank::BankE, &self.vramcnt[offset as usize]),
        BANK_F => self.vram.map_bank(Bank::BankF, &self.vramcnt[offset as usize]),
        BANK_G => self.vram.map_bank(Bank::BankG, &self.vramcnt[offset as usize]),
        BANK_H => self.vram.map_bank(Bank::BankH, &self.vramcnt[offset as usize]),
        BANK_I => self.vram.map_bank(Bank::BankI, &self.vramcnt[offset as usize]),
        _ => todo!("unimplemented")
      }
    }
  }

  pub fn read_vramcnt(&self, offset: u32) -> u8 {
    self.vramcnt[offset as usize].read()
  }

  pub fn get_arm7_vram_stat(&self) -> u8 {
    ((self.vramcnt[2].vram_enable && self.vramcnt[2].vram_mst == 2) as u8) | ((self.vramcnt[3].vram_enable && self.vramcnt[3].vram_mst == 2) as u8) << 1
  }

  pub fn schedule_hdraw(&mut self, scheduler: &mut Scheduler, cycles_left: usize) {
    scheduler.schedule(EventType::HDraw, HDRAW_CYCLES - cycles_left);
  }

  fn trigger_vblank(&mut self) {
    for dispstat in &mut self.dispstat {
      dispstat.flags.insert(DispStatFlags::VBLANK);
    }
  }

  fn render_line(&mut self) {
    if self.powcnt1.contains(PowerControlRegister1::ENGINE_A_ENABLE) {
      self.engine_a.render_line(self.vcount, &mut self.vram, &self.engine3d.frame_buffer);

      // capture image if needed
      if self.is_capturing && self.vcount < self.dispcapcnt.get_capture_height() {
        self.dispcapcnt.capture_enable = false;
        self.start_capture_image();
      }
    }
    if self.powcnt1.contains(PowerControlRegister1::ENGINE_B_ENABLE) {
      self.engine_b.render_line(self.vcount, &mut self.vram, &self.engine3d.frame_buffer);
    }
  }
}