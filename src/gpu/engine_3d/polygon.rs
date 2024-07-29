use super::{polygon_attributes::PolygonAttributes, texture_params::TextureParams, PrimitiveType};

#[derive(Debug)]
pub struct Polygon {
  pub start: usize,
  pub end: usize,
  pub attributes: PolygonAttributes,
  pub palette_base: usize,
  pub is_front: bool,
  pub tex_params: TextureParams,
  pub top: u32,
  pub bottom: u32,
  pub primitive_type: PrimitiveType
}