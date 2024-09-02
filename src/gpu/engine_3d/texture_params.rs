
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TextureFormat {
  None,
  A3I5Translucent,
  Color4,
  Color16,
  Color256,
  Color4x4,
  A5I3Translucent,
  Direct
}


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TransformationMode {
  None = 0,
  TexCoord = 1,
  Normal = 2,
  Vertex = 3
}


#[derive(Copy, Clone, Debug)]
pub struct TextureParams {
  pub vram_offset: u32,
  pub texture_s_size: u32,
  pub texture_t_size: u32,
  pub size_s_shift: u32,
  pub size_t_shift: u32,
  pub texture_format: TextureFormat,
  pub transformation_mode: TransformationMode,
  pub repeat_s: bool,
  pub repeat_t: bool,
  pub flip_s: bool,
  pub flip_t: bool,
  pub color0_transparent: bool
}

impl TextureParams {
  pub fn new() -> Self {
    Self {
      vram_offset: 0,
      texture_format: TextureFormat::None,
      texture_s_size: 0,
      texture_t_size: 0,
      size_s_shift: 0,
      size_t_shift: 0,
      transformation_mode: TransformationMode::None,
      repeat_s: false,
      repeat_t: false,
      flip_s: false,
      flip_t: false,
      color0_transparent: false
    }
  }

  pub fn write(&mut self, value: u32) {
    self.vram_offset = (value & 0xffff) << 3;
    self.texture_s_size = 8 << (value >> 20 & 0x7);
    self.texture_t_size = 8 << (value >> 23 & 0x7);
    self.size_s_shift = 3 + (value >> 20 & 0x7);
    self.size_t_shift = 3 + (value >> 23 & 0x7);

    self.texture_format = match value >> 26 & 0x7 {
      0 => TextureFormat::None,
      1 => TextureFormat::A3I5Translucent,
      2 => TextureFormat::Color4,
      3 => TextureFormat::Color16,
      4 => TextureFormat::Color256,
      5 => TextureFormat::Color4x4,
      6 => TextureFormat::A5I3Translucent,
      7 => TextureFormat::Direct,
      _ => unreachable!()
    };

    self.transformation_mode = match value >> 30 & 0x3 {
      0 => TransformationMode::None,
      1 => TransformationMode::TexCoord,
      2 => TransformationMode::Normal,
      3 => TransformationMode::Vertex,
      _ => unreachable!()
    };

    self.repeat_s = (value >> 16) & 0b1 == 1;
    self.repeat_t = (value >> 17) & 0b1 == 1;
    self.flip_s = (value >> 18) & 0b1 == 1;
    self.flip_t = (value >> 19) & 0b1 == 1;
    self.color0_transparent = (value >> 29) & 0b1 == 1;

  }
}