#[derive(Copy, Clone)]
pub enum CaptureSize {
  Size128 = 0,
  Size256by64 = 1,
  Size256by128 = 2,
  Size256by192 = 3
}

#[derive(Copy, Clone, PartialEq)]
pub enum ScreenSourceA {
  GraphicsScreen = 0,
  Screen3d = 1
}

#[derive(Copy, Clone, PartialEq)]
pub enum CaptureSource {
  SourceA = 0,
  SourceB = 1,
  Blended = 2
}


#[derive(Copy, Clone, PartialEq)]
pub enum ScreenSourceB {
  VRam,
  MainMemoryDisplayFifo
}

pub struct DisplayCaptureControlRegister {
  pub eva: u32,
  pub evb: u32,
  pub vram_write_block: u32,
  pub vram_write_offset: u32,
  pub capture_size: CaptureSize,
  pub source_a: ScreenSourceA,
  pub source_b: ScreenSourceB,
  pub vram_read_offset: u32,
  pub capture_source: CaptureSource,
  pub capture_enable: bool
}

impl DisplayCaptureControlRegister {
  pub fn new() -> Self {
    Self {
      eva: 0,
      evb: 0,
      vram_read_offset: 0,
      vram_write_block: 0,
      vram_write_offset: 0,
      source_a: ScreenSourceA::GraphicsScreen,
      source_b: ScreenSourceB::VRam,
      capture_size: CaptureSize::Size128,
      capture_enable: false,
      capture_source: CaptureSource::SourceA
    }
  }

  pub fn get_capture_height(&self) -> u16 {
    match self.capture_size {
      CaptureSize::Size128 | CaptureSize::Size256by128 => 128,
      CaptureSize::Size256by192 => 192,
      CaptureSize::Size256by64 => 64
    }
  }

  pub fn get_capture_width(&self) -> u16 {
    match self.capture_size {
      CaptureSize::Size128 => 128,
      CaptureSize::Size256by192 | CaptureSize::Size256by128 | CaptureSize::Size256by64 => 256
    }
  }

  pub fn read(&self) -> u32 {
    self.eva |
    self.evb << 5 |
    self.vram_write_block << 16 |
    self.vram_write_offset << 18 |
    (self.capture_size as u32) << 20 |
    (self.source_a as u32) << 24 |
    (self.source_b as u32) << 25 |
    self.vram_read_offset << 26 |
    (self.capture_source as u32) << 29 |
    (self.capture_enable as u32) << 31
  }
  /*
  0-4   EVA               (0..16 = Blending Factor for Source A)
  5-7   Not used
  8-12  EVB               (0..16 = Blending Factor for Source B)
  13-15 Not used
  16-17 VRAM Write Block  (0..3 = VRAM A..D) (VRAM must be allocated to LCDC)
  18-19 VRAM Write Offset (0=00000h, 0=08000h, 0=10000h, 0=18000h)
  20-21 Capture Size      (0=128x128, 1=256x64, 2=256x128, 3=256x192 dots)
  22-23 Not used
  24    Source A          (0=Graphics Screen BG+3D+OBJ, 1=3D Screen)
  25    Source B          (0=VRAM, 1=Main Memory Display FIFO)
  26-27 VRAM Read Offset  (0=00000h, 0=08000h, 0=10000h, 0=18000h)
  28    Not used
  29-30 Capture Source    (0=Source A, 1=Source B, 2/3=Sources A+B blended)
  31    Capture Enable    (0=Disable/Ready, 1=Enable/Busy)
  */
  pub fn write(&mut self, val: u32) {
    self.eva = val & 0x1f;
    self.evb = (val >> 8) & 0x1f;
    self.vram_write_block = (val >> 16) & 0x3;
    self.vram_write_offset = (val >> 18) & 0x3;
    self.capture_size = match (val >> 20) & 0x3 {
      0 => CaptureSize::Size128,
      1 => CaptureSize::Size256by64,
      2 => CaptureSize::Size256by128,
      3 => CaptureSize::Size256by192,
      _ => unreachable!()
    };

    self.source_a = match (val >> 24) & 0x1 {
      0 => ScreenSourceA::GraphicsScreen,
      1 => ScreenSourceA::Screen3d,
      _ => unreachable!()
    };

    self.source_b = match (val >> 25) & 0x1 {
      0 => ScreenSourceB::VRam,
      1 => ScreenSourceB::MainMemoryDisplayFifo,
      _ => unreachable!()
    };

    self.vram_read_offset = (val >> 26) & 0x3;

    self.capture_source = match (val >> 29) & 0x3 {
      0 => CaptureSource::SourceA,
      1 => CaptureSource::SourceB,
      2 => CaptureSource::Blended,
      _ => unreachable!()
    };

    self.capture_enable = (val >> 31) & 0x1 == 1;
  }
}