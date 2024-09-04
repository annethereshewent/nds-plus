/*
  0     Texture Mapping      (0=Disable, 1=Enable)
  1     PolygonAttr Shading  (0=Toon Shading, 1=Highlight Shading)
  2     Alpha-Test           (0=Disable, 1=Enable) (see ALPHA_TEST_REF)
  3     Alpha-Blending       (0=Disable, 1=Enable) (see various Alpha values)
  4     Anti-Aliasing        (0=Disable, 1=Enable)
  5     Edge-Marking         (0=Disable, 1=Enable) (see EDGE_COLOR)
  6     Fog Color/Alpha Mode (0=Alpha and Color, 1=Only Alpha) (see FOG_COLOR)
  7     Fog Master Enable    (0=Disable, 1=Enable)
  8-11  Fog Depth Shift      (FOG_STEP=400h shr FOG_SHIFT) (see FOG_OFFSET)
  12    Color Buffer RDLINES Underflow (0=None, 1=Underflow/Acknowledge)
  13    Polygon/Vertex RAM Overflow    (0=None, 1=Overflow/Acknowledge)
  14    Rear-Plane Mode                (0=Blank, 1=Bitmap)
*/

bitflags! {
  pub struct Display3dControlRegister: u32 {
    const TEXTURE_MAPPING_ENABLE = 1;
    const POLYGON_ATTR_SHADING = 1 << 1;
    const ALPHA_TEST_ENABLE = 1 << 2;
    const ALPHA_BLENDING_ENABLE = 1 << 3;
    const ANTI_ALIASING_ENABLE = 1 << 4;
    const EDGE_MARKING_ENABLE = 1 << 5;
    const FOG_ALPHA_ONLY = 1 << 6;
    const FOG_MASTER_ENABLE = 1 << 7;
    const COLOR_BUFFER_UNDERFLOW_ACKNOWLEDGE = 1 << 12;
    const POLYGON_OVERFLOW_ACKNOWLEDGE = 1 << 13;
    const REAR_PLANE_MODE = 1 << 14;
  }
}

impl Display3dControlRegister {
  pub fn fog_depth_shift(&self) -> u32 {
    (self.bits() >> 8) & 0xf
  }
}