#[derive(Debug)]
pub enum PolygonMode {
  Modulation = 0,
  Decal = 1,
  Toon = 2,
  Shadow = 3
}


bitflags! {
  #[derive(Copy, Clone, Debug)]
  pub struct PolygonAttributes: u32 {
    const SHOW_BACK_SURFACE = 1 << 6;
    const SHOW_FRONT_SURFACE = 1 << 7;
    const UPDATE_DEPTH_FOR_TRANSLUCENT = 1 << 11;
    const CLIP_FAR_PLANE = 1 << 12;
    const RENDER_1_DOT = 1 << 13;
    const DRAW_PIXELS_WITH_DEPTH = 1 << 14;
    const FOG_ENABLE = 1 << 15;
  }
}

impl PolygonAttributes {
  pub fn light_enabled(&self, id: usize) -> bool {
    self.bits() >> id & 0b1 == 1
  }

  pub fn alpha(&self) -> u8 {
    ((self.bits() >> 16) & 0x1f) as u8
  }

  pub fn polygon_id(&self) -> u32 {
    (self.bits() >> 24) & 0x3f
  }

  pub fn polygon_mode(&self) -> PolygonMode {
    match (self.bits() >> 4) & 0x3 {
      0 => PolygonMode::Modulation,
      1 => PolygonMode::Decal,
      2 => PolygonMode::Toon,
      3 => PolygonMode::Shadow,
      _ => unreachable!()
    }
  }
}