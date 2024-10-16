use serde::{Deserialize, Serialize};

bitflags! {
  #[derive(Serialize, Deserialize)]
  #[serde(transparent)]
  pub struct PowerControlRegister1: u16 {
    const LCD_ENABLE = 1;
    const ENGINE_A_ENABLE = 1 << 1;
    const ENGINE_3D_ENABLE = 1 << 2;
    const ENGINE_3D_GEOMETRY_ENABLE = 1 << 3;
    const ENGINE_B_ENABLE = 1 << 9;
    const TOP_A = 1 << 15;
  }
}