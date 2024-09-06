#[derive(Clone, Copy, Debug)]
pub struct RenderingAttributes {
  pub is_translucent: bool,
  pub front_facing: bool,
  pub fog_enabled: bool
}

impl RenderingAttributes {
  pub fn new() -> Self {
    Self {
      is_translucent: false,
      front_facing: false,
      fog_enabled: false
    }
  }
}