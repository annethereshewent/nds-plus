use super::{color::Color, engine_3d::{polygon::Polygon, vertex::Vertex}, registers::{clear_color_register::ClearColorRegister, display_3d_control_register::Display3dControlRegister, geometry_status_register::GeometryStatusRegister}};

pub struct RenderingData3d {
  pub clear_color: ClearColorRegister,
  pub clear_depth: u32,
  pub vertices_buffer: Vec<Vertex>,
  pub polygon_buffer: Vec<Polygon>,
  pub toon_table: [Color; 32],
  pub disp3dcnt: Display3dControlRegister,
  pub gxstat: GeometryStatusRegister,
}

impl RenderingData3d {
  pub fn new() -> Self {
    Self {
      clear_color: ClearColorRegister::new(),
      clear_depth: 0,
      vertices_buffer: Vec::new(),
      polygon_buffer: Vec::new(),
      toon_table: [Color::new(); 32],
      disp3dcnt: Display3dControlRegister::from_bits_retain(0),
      gxstat: GeometryStatusRegister::new(),
    }
  }
}