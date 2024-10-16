use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Texcoord {
  pub u: i16,
  pub v: i16
}

impl Texcoord {
  pub fn new() -> Self {
    Self {
      u: 0,
      v: 0
    }
  }
}