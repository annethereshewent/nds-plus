use std::sync::Arc;

use crate::gpu::{ThreadData, SCREEN_WIDTH};

use super::ObjectPixel;

pub struct Renderer2d {
  pub thread_data: Arc<ThreadData>,
  pub obj_lines: [ObjectPixel; SCREEN_WIDTH as usize],
}

impl Renderer2d {

}