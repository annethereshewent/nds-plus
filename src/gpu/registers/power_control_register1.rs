bitflags! {
  pub struct PowerControlRegister1: u32 {
    const LCD_ENABLE = 1;
    const ENGINE_A_ENABLE = 1 << 1;
    const ENGINE_3D_ENABLE = 1 << 2;
    const ENGINE_3D_GEOMETRY_ENABLE = 1 << 3;
    const ENGINE_B_ENABLE = 1 << 9;
    const DISPLAY_SWAP = 1 << 15;
  }
}