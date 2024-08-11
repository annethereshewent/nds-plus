#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BgMode {
  Mode0 = 0,
  Mode1 = 1,
  Mode2 = 2,
  Mode3 = 3,
  Mode4 = 4,
  Mode5 = 5,
  Mode6 = 6
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DisplayMode {
  Mode0 = 0,
  Mode1 = 1,
  Mode2 = 2,
  Mode3 = 3
}

#[derive(Debug, Copy, Clone)]
pub struct DisplayControlRegister {
  pub flags: DisplayControlRegisterFlags,
  pub bg_mode: BgMode,
  pub display_mode: DisplayMode,
  pub vram_block: u32,
  pub tile_obj_boundary: u32,
  pub character_base: u32,
  pub screen_base: u32,
  value: u32
}

impl DisplayControlRegister {
  pub fn new() -> Self {
    Self {
      flags: DisplayControlRegisterFlags::from_bits_retain(0),
      bg_mode: BgMode::Mode0,
      display_mode: DisplayMode::Mode0,
      vram_block: 0,
      tile_obj_boundary: 0,
      character_base: 0,
      screen_base: 0,
      value: 0
    }
  }

  pub fn write(&mut self, val: u32, mask: Option<u32>, is_engine_b: bool) {
    let mut value = 0;

    if let Some(mask) = mask {
      value = self.value & mask;
    }

    value |= val;

    self.value = value;

    self.flags = DisplayControlRegisterFlags::from_bits_retain(value);

    self.bg_mode = match value & 0x7 {
      0 => BgMode::Mode0,
      1 => BgMode::Mode1,
      2 => BgMode::Mode2,
      3 => BgMode::Mode3,
      4 => BgMode::Mode4,
      5 => BgMode::Mode5,
      6 => BgMode::Mode6,
      _ => panic!("unknown bg mode received")
    };

    self.display_mode = match (value >> 16) & 0b11 {
      0 => DisplayMode::Mode0,
      1 => DisplayMode::Mode1,
      2 => DisplayMode::Mode2,
      3 => DisplayMode::Mode3,
      _ => unreachable!("can't happen")
    };

    self.tile_obj_boundary = (value >> 20) & 0x3;

    if !is_engine_b {
      self.vram_block = (value >> 18) & 0x3;
      self.character_base = (value >> 24) & 0x7;
      self.screen_base = (value >> 27) & 0x7;
    } else {
      // these only exist on engine a
      self.flags.remove(DisplayControlRegisterFlags::BG_3D_SELECTION);
      self.flags.remove(DisplayControlRegisterFlags::BITMAP_OBJ_1D_BOUNDARY);
    }
  }

  pub fn read(&self) -> u32 {
    self.value
  }

  pub fn windows_enabled(&self) -> bool {
    self.flags.contains(DisplayControlRegisterFlags::DISPLAY_WINDOW0) || self.flags.contains(DisplayControlRegisterFlags::DISPLAY_WINDOW1)
  }

}

bitflags! {
  #[derive(Debug, Copy, Clone)]
  pub struct DisplayControlRegisterFlags: u32 {
    const BG_3D_SELECTION = 1 << 3;
    const TILE_OBJ_MAPPINGS = 1 << 4;
    const BITMAP_OBJ_2D_DIMENSION = 1 << 5;
    const BITMAP_OBJ_MAPPING = 1 << 6;
    const FORCED_BLANK = 1 << 7;
    const DISPLAY_BG0 = 1 << 8;
    const DISPLAY_BG1 = 1 << 9;
    const DISPLAY_BG2 = 1 << 10;
    const DISPLAY_BG3 = 1 << 11;
    const DISPLAY_OBJ = 1 << 12;
    const DISPLAY_WINDOW0 = 1 << 13;
    const DISPLAY_WINDOW1 = 1 << 14;
    const DISPLAY_OBJ_WINDOW = 1 << 15;
    const BITMAP_OBJ_1D_BOUNDARY = 1 << 22;
    const OBJ_PROCESSING_DURING_HBLANK = 1 << 23;
    const BG_EXTENDED_PALETTES = 1 << 30;
    const OBJ_EXTENDED_PALETTES = 1 << 31;
  }
}