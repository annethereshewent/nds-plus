pub enum TextureFormat {
  None,
  A315Transluscent,
  Color4,
  Color16,
  Color256,
  Color4x4,
  A513Transluscent,
  Direct
}

pub enum TransformationMode {
  None = 0,
  TexCoord = 1,
  Normal = 2,
  Vertex = 3
}

bitflags! {
  pub struct TextureParams: u32 {
    const REPEAT_S = 1 << 16;
    const REPEAT_T = 1 << 17;
    const FLIP_S = 1 << 18;
    const FLIP_T = 1 << 19;
    const DISPLAY_COLOR_0 = 1 << 29;
  }
}

impl TextureParams {
  pub fn vram_offset(&self) -> u32 {
    self.bits() & 0xffff
  }

  pub fn texture_s_size(&self) -> u32 {
    self.bits() >> 20 & 0x7
  }

  pub fn texture_t_size(&self) -> u32 {
    self.bits() >> 23 & 0x7
  }

  pub fn texture_format(&self) -> TextureFormat {
    match self.bits() >> 26 & 0x7 {
      0 => TextureFormat::None,
      1 => TextureFormat::A315Transluscent,
      2 => TextureFormat::Color4,
      3 => TextureFormat::Color16,
      4 => TextureFormat::Color256,
      5 => TextureFormat::Color4x4,
      6 => TextureFormat::A513Transluscent,
      7 => TextureFormat::Direct,
      _ => unreachable!()
    }
  }

  pub fn transformation_mode(&self) -> TransformationMode {
    match self.bits() >> 30 & 0x3 {
      0 => TransformationMode::None,
      1 => TransformationMode::TexCoord,
      2 => TransformationMode::Normal,
      3 => TransformationMode::Vertex,
      _ => unreachable!()
    }
  }
}