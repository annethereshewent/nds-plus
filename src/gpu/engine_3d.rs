use std::collections::{HashSet, VecDeque};

use box_test::BoxTest;
use diffuse_color::DiffuseColor;
use light::Light;
use matrix::Matrix;
use polygon::Polygon;
use polygon_attributes::PolygonAttributes;
use specular_color::SpecularColor;
use texcoord::Texcoord;
use texture_params::{TextureParams, TransformationMode};
use vertex::Vertex;
use viewport::Viewport;

use crate::cpu::registers::interrupt_request_register::InterruptRequestRegister;

use super::{
  color::Color,
  registers::{
    clear_color_register::ClearColorRegister, display_3d_control_register::Display3dControlRegister, fog_color_register::FogColorRegister, geometry_status_register::{GeometryIrq, GeometryStatusRegister}
  }, SCREEN_HEIGHT, SCREEN_WIDTH
};

pub mod matrix;
pub mod polygon_attributes;
pub mod texture_params;
pub mod rendering3d;
pub mod viewport;
pub mod diffuse_color;
pub mod specular_color;
pub mod light;
pub mod vertex;
pub mod texcoord;
pub mod polygon;
pub mod box_test;

pub const FIFO_CAPACITY: usize = 256;
pub const POLYGON_BUFFER_SIZE: usize = 2048;
pub const VERTEX_BUFFER_SIZE: usize = 6144;

#[derive(Copy, Clone, Debug)]
pub struct Pixel3d {
  pub color: Option<Color>,
  pub depth: u32,
  pub alpha: u8
}

impl Pixel3d {
  pub fn new() -> Self {
    Self {
      color: None,
      depth: 0,
      alpha: 0
    }
  }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PrimitiveType {
  Triangles,
  Quads,
  QuadStrips,
  TriangleStrips
}

impl PrimitiveType {
  pub fn from(value: u32) -> Self {

    match value {
      0 => PrimitiveType::Triangles,
      1 => PrimitiveType::Quads,
      2 => PrimitiveType::TriangleStrips,
      3 => PrimitiveType::QuadStrips,
      _ => unreachable!()
    }
  }

  pub fn get_num_vertices(&self) -> usize {
    match self {
      Self::Triangles | Self::TriangleStrips => 3,
      Self::Quads | Self::QuadStrips => 4
    }
  }
}

#[derive(Copy, Clone, PartialEq)]
enum MatrixMode {
  Projection,
  Position,
  PositionAndVector,
  Texture
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Command {
  Nop,
  MtxMode,
  MtxIdentity,
  MtxLoad4x4,
  MtxLoad4x3,
  MtxMult4x4,
  MtxMult4x3,
  MtxMult3x3,
  MtxScale,
  MtxTrans,
  MtxPush,
  MtxPop,
  MtxStore,
  MtxRestore,
  PolygonAttr,
  Color,
  BeginVtxs,
  EndVtxs,
  Vtx16,
  Vtx10,
  VtxXy,
  VtxXz,
  VtxYz,
  VtxDiff,
  LightVector,
  LightColor,
  DifAmb,
  SpeEmi,
  Shininess,
  Normal,
  Texcoord,
  TexImageParam,
  PlttBase,
  BoxTest,
  PosTest,
  VecTest,
  SwapBuffers,
  Viewport
}

impl Command {
  pub fn from(value: u8) -> Self {
    use Command::*;
    match value {
      0x00 => Nop,
      0x10 => MtxMode,
      0x11 => MtxPush,
      0x12 => MtxPop,
      0x13 => MtxStore,
      0x14 => MtxRestore,
      0x15 => MtxIdentity,
      0x16 => MtxLoad4x4,
      0x17 => MtxLoad4x3,
      0x18 => MtxMult4x4,
      0x19 => MtxMult4x3,
      0x1a => MtxMult3x3,
      0x1b => MtxScale,
      0x1c => MtxTrans,
      0x20 => Color,
      0x21 => Normal,
      0x22 => Texcoord,
      0x23 => Vtx16,
      0x24 => Vtx10,
      0x25 => VtxXy,
      0x26 => VtxXz,
      0x27 => VtxYz,
      0x28 => VtxDiff,
      0x29 => PolygonAttr,
      0x2a => TexImageParam,
      0x2b => PlttBase,
      0x30 => DifAmb,
      0x31 => SpeEmi,
      0x32 => LightVector,
      0x33 => LightColor,
      0x34 => Shininess,
      0x40 => BeginVtxs,
      0x41 => EndVtxs,
      0x50 => SwapBuffers,
      0x60 => Viewport,
      0x70 => BoxTest,
      0x71 => PosTest,
      0x72 => VecTest,
      _ => panic!("unrecognized command received: {:x}", value)
    }
  }

  pub fn from_address(address: u32) -> Self {
    use Command::*;
    match address {
      0x440 => MtxMode,
      0x444 => MtxPush,
      0x448 => MtxPop,
      0x44c => MtxStore,
      0x450 => MtxRestore,
      0x454 => MtxIdentity,
      0x458 => MtxLoad4x4,
      0x45c => MtxLoad4x3,
      0x460 => MtxMult4x4,
      0x464 => MtxMult4x3,
      0x468 => MtxMult3x3,
      0x46c => MtxScale,
      0x470 => MtxTrans,
      0x480 => Color,
      0x484 => Normal,
      0x488 => Texcoord,
      0x48c => Vtx16,
      0x490 => Vtx10,
      0x494 => VtxXy,
      0x498 => VtxXz,
      0x49c => VtxYz,
      0x4a4 => PolygonAttr,
      0x4a8 => TexImageParam,
      0x4ac => PlttBase,
      0x4c0 => DifAmb,
      0x4c4 => SpeEmi,
      0x4c8 => LightVector,
      0x4cc => LightColor,
      0x4d0 => Shininess,
      0x500 => BeginVtxs,
      0x504 => EndVtxs,
      0x540 => SwapBuffers,
      0x580 => Viewport,
      0x5c0 => BoxTest,
      0x5c8 => VecTest,
      _ => panic!("invalid address given to Command::from_address: {:x}", address)
    }
  }

  pub fn get_num_params(&self) -> usize {
    use Command::*;
    match *self {
      Nop => 0,
      MtxMode => 1,
      MtxPush => 0,
      MtxPop => 1,
      MtxStore => 1,
      MtxRestore => 1,
      MtxIdentity => 0,
      MtxLoad4x4 => 16,
      MtxLoad4x3 => 12,
      MtxMult4x4 => 16,
      MtxMult4x3 => 12,
      MtxMult3x3 => 9,
      MtxScale => 3,
      MtxTrans => 3,
      Color => 1,
      Normal => 1,
      Texcoord => 1,
      Vtx16 => 2,
      Vtx10 => 1,
      VtxXy => 1,
      VtxXz => 1,
      VtxYz => 1,
      VtxDiff => 1,
      PolygonAttr => 1,
      TexImageParam => 1,
      PlttBase => 1,
      DifAmb => 1,
      SpeEmi => 1,
      LightVector => 1,
      LightColor => 1,
      Shininess => 32,
      BeginVtxs => 1,
      EndVtxs => 0,
      SwapBuffers => 1,
      Viewport => 1,
      BoxTest => 3,
      PosTest => 2,
      VecTest => 1
    }
  }
}

#[derive(Copy, Clone, Debug)]
pub struct GeometryCommandEntry {
  command: Command,
  param: u32
}

impl GeometryCommandEntry {
  pub fn new() -> Self {
    Self {
      command: Command::Nop,
      param: 0
    }
  }

  pub fn from(command: Command, param: u32) -> Self {
    Self {
      command,
      param
    }
  }
}

pub struct Engine3d {
  fifo: VecDeque<GeometryCommandEntry>,
  packed_commands: u32,
  current_command: Command,
  params_processed: usize,
  num_params: usize,
  gxstat: GeometryStatusRegister,
  clear_color: ClearColorRegister,
  clear_depth: u32,
  clear_offset_x: u16,
  clear_offset_y: u16,
  fog_color: FogColorRegister,
  fog_offset: u16,
  fog_table: [u8; 32],
  edge_colors: [Color; 8],
  toon_table: [Color; 32],
  shininess_table: [u8; 128],
  matrix_mode: MatrixMode,
  current_position_matrix: Matrix,
  current_vector_matrix: Matrix,
  current_projection_matrix: Matrix,
  current_texture_matrix: Matrix,
  position_vector_sp: u8,
  projection_sp: u8,
  texture_sp: u8,
  texture_stack: Matrix,
  position_stack: [Matrix; 32],
  vector_stack: [Matrix; 32],
  projection_stack: Matrix,
  command_started: bool,
  command_params: usize,
  polygon_attributes: PolygonAttributes,
  internal_polygon_attributes: PolygonAttributes,
  texture_params: TextureParams,
  palette_base: u32,
  transluscent_polygon_sort: bool,
  depth_buffering_with_w: bool,
  polygons_ready: bool,
  viewport: Viewport,
  temp_matrix: Matrix,
  diffuse_reflection: DiffuseColor,
  ambient_reflection: Color,
  specular_reflection: SpecularColor,
  emission: Color,
  lights: [Light; 4],
  vertex_color: Color,
  vec_result: [i32; 4],
  primitive_type: PrimitiveType,
  current_vertices: Vec<Vertex>,
  translation_vector: [i32; 3],
  texcoord: Texcoord,
  original_texcoord: Texcoord,
  current_vertex: Vertex,
  max_vertices: usize,
  clip_mtx_recalculate: bool,
  clip_matrix: Matrix,
  vertices_buffer: Vec<Vertex>,
  polygon_buffer: Vec<Polygon>,
  scale_vector: [i32; 3],
  pub frame_buffer: [Pixel3d; SCREEN_HEIGHT as usize * SCREEN_WIDTH as usize],
  alpha_ref: u8,
  max_params: usize,
  swap_vertices: bool,
  pub disp3dcnt: Display3dControlRegister,
  pub debug_on: bool,
  box_test: BoxTest,
  pub found: HashSet<String>
}

impl Engine3d {
  pub fn new() -> Self {
    Self {
      fifo: VecDeque::with_capacity(256),
      packed_commands: 0,
      current_command: Command::Nop,
      params_processed: 0,
      num_params: 0,
      gxstat: GeometryStatusRegister::new(),
      clear_color: ClearColorRegister::new(),
      clear_depth: 0,
      clear_offset_x: 0,
      clear_offset_y: 0,
      fog_color: FogColorRegister::new(),
      fog_offset: 0,
      edge_colors:  [Color::new(); 8],
      toon_table: [Color::new(); 32],
      shininess_table: [0; 128],
      fog_table: [0; 32],
      matrix_mode: MatrixMode::Projection,
      current_position_matrix: Matrix::new(),
      current_projection_matrix: Matrix::new(),
      current_vector_matrix: Matrix::new(),
      current_texture_matrix: Matrix::new(),
      projection_stack: Matrix::new(),
      position_stack: Matrix::create_vector_position_stack(),
      vector_stack: Matrix::create_vector_position_stack(),
      texture_stack: Matrix::new(),
      position_vector_sp: 0,
      projection_sp: 0,
      texture_sp: 0,
      command_started: false,
      command_params: 0,
      polygon_attributes: PolygonAttributes::from_bits_retain(0),
      internal_polygon_attributes: PolygonAttributes::from_bits_retain(0),
      texture_params: TextureParams::new(),
      palette_base: 0,
      transluscent_polygon_sort: false,
      depth_buffering_with_w: false,
      polygons_ready: false,
      viewport: Viewport::new(),
      temp_matrix: Matrix::new(),
      diffuse_reflection: DiffuseColor::new(),
      ambient_reflection: Color::new(),
      emission: Color::new(),
      specular_reflection: SpecularColor::new(),
      lights: [Light::new(); 4],
      vertex_color: Color::new(),
      vec_result: [0; 4],
      primitive_type: PrimitiveType::Triangles,
      current_vertices: Vec::new(),
      translation_vector: [0; 3],
      scale_vector: [0; 3],
      texcoord: Texcoord::new(),
      original_texcoord: Texcoord::new(),
      current_vertex: Vertex::new(),
      max_vertices: 0,
      clip_mtx_recalculate: false,
      clip_matrix: Matrix::new(),
      vertices_buffer: Vec::new(),
      polygon_buffer: Vec::new(),
      frame_buffer: [Pixel3d::new(); SCREEN_HEIGHT as usize * SCREEN_WIDTH as usize],
      alpha_ref: 0,
      max_params: 0,
      swap_vertices: false,
      disp3dcnt: Display3dControlRegister::from_bits_retain(0),
      debug_on: false,
      box_test: BoxTest::new(),
      found: HashSet::new()
    }
  }

  pub fn clear_frame_buffer(&mut self) {
    for pixel in &mut self.frame_buffer {
      *pixel = Pixel3d::new();
      pixel.depth = self.clear_depth as u32;
    }
  }

  pub fn read_ram_count(&self) -> u16 {
    self.polygon_buffer.len() as u16 | (self.vertices_buffer.len() << 16) as u16
  }

  pub fn read_clip_matrix(&mut self, address: u32) -> u32 {
    if self.clip_mtx_recalculate {
      self.recalculate_clip_matrix();
    }

    let index = (address - 0x400_0640) / 4;

    let row = index / 4;
    let column = index % 4;

    self.clip_matrix.0[row as usize][column as usize] as u32
  }

  pub fn read_vector_matrix(&mut self, address: u32) -> u32 {
    let index = (address - 0x400_0680) / 4;

    let row = index / 4;
    let column = index % 4;

    self.current_vector_matrix.0[row as usize][column as usize] as u32
  }

  pub fn read_geometry_status(&mut self, interrupt_request: &mut InterruptRequestRegister) -> u32 {
    let value = self.gxstat.read(self.position_vector_sp as u32, self.projection_sp as u32, &self.fifo);

    self.check_interrupts(interrupt_request);

    value
  }

  pub fn write_alpha_ref(&mut self, value: u16) {
    self.alpha_ref = (value & 0x1f) as u8;
  }

  pub fn write_fog_table(&mut self, address: u32, value: u8) {
    let offset = address - 0x400_0360;

    self.fog_table[offset as usize] = value & 0x7f;
  }

  pub fn write_edge_color(&mut self, address: u32, value: u16) {
    let offset = (address - 0x400_0330) / 2;

    self.edge_colors[offset as usize].write(value);
  }

  pub fn write_toon_table(&mut self, address: u32, value: u16) {
    let offset = (address - 0x400_0380) / 2;

    self.toon_table[offset as usize].write(value);
  }

  pub fn write_fog_color(&mut self, value: u32) {
    self.fog_color.write(value);
  }

  pub fn write_fog_offset(&mut self, value: u16) {
    self.fog_offset = value & 0x7fff;
  }

  pub fn write_clear_color(&mut self, value: u32) {
    self.clear_color.write(value);
  }

  pub fn write_clear_depth(&mut self, value: u16) {
    self.clear_depth = (value as u32 & 0x7fff) * 0x200 + 0x1ff;
  }

  pub fn write_clear_image_offset(&mut self, value: u16) {
    self.clear_offset_x = value & 0xff;
    self.clear_offset_y = (value >> 8) & 0xff;
  }

  pub fn write_geometry_status(&mut self, value: u32, interrupt_request: &mut InterruptRequestRegister, mask: Option<u32>) {
    let mut val = 0;

    if let Some(mask) = mask {
      val = self
        .gxstat
        .read(
          self.position_vector_sp as u32,
          self.projection_sp as u32,
          &self.fifo
        ) & mask;
    }

    val |= value;

    self.gxstat.write(val);

    self.check_interrupts(interrupt_request);
  }

  pub fn write_geometry_command(&mut self, address: u32, value: u32, interrupt_request: &mut InterruptRequestRegister) {
    let command = Command::from_address(address & 0xfff);

    self.push_command(GeometryCommandEntry::from(command, value), interrupt_request);
  }

  pub fn execute_commands(&mut self, interrupt_request: &mut InterruptRequestRegister) {
    if !self.polygons_ready {
      while let Some(entry) = self.fifo.pop_front() {
        self.execute_command(entry);

        if self.polygons_ready {
          break;
        }
      }
    }

    self.check_interrupts(interrupt_request);
  }

  pub fn should_run_dmas(&self) -> bool {
    !self.polygons_ready && self.fifo.len() < FIFO_CAPACITY / 2
  }

  pub fn check_interrupts(&mut self, interrupt_request: &mut InterruptRequestRegister) {
    match self.gxstat.geometry_irq {
      GeometryIrq::Empty => if self.fifo.is_empty() {
        interrupt_request.insert(InterruptRequestRegister::GEOMETRY_COMMAND);
      }
      GeometryIrq::LessThanHalfFull => if self.fifo.len() < FIFO_CAPACITY / 2 {
        interrupt_request.insert(InterruptRequestRegister::GEOMETRY_COMMAND);
      }
      _ => ()
    }
  }

  fn execute_command(&mut self, entry: GeometryCommandEntry) {
    if self.fifo.len() < FIFO_CAPACITY {
      self.gxstat.geometry_engine_busy = false;
    }

    use Command::*;
    match entry.command {
      EndVtxs => (), // just a NOP
      MtxMode => {
        self.matrix_mode = match entry.param & 0x3 {
          0 => MatrixMode::Projection,
          1 => MatrixMode::Position,
          2 => MatrixMode::PositionAndVector,
          3 => MatrixMode::Texture,
          _ => unreachable!()
        };
      }
      MtxIdentity => {
        match self.matrix_mode {
          MatrixMode::Position => {
            self.current_position_matrix = Matrix::new();
            self.clip_mtx_recalculate = true;
          }
          MatrixMode::PositionAndVector => {
            self.current_position_matrix = Matrix::new();
            self.current_vector_matrix = Matrix::new();

            self.clip_mtx_recalculate = true;
          }
          MatrixMode::Projection => {
            self.current_projection_matrix = Matrix::new();
            self.clip_mtx_recalculate = true;
          }
          MatrixMode::Texture => {
            self.current_texture_matrix = Matrix::new();
          }
        }
      }
      MtxPop => {
        let offset =((entry.param & 0x3f) as i8) << 2 >> 2;
        match self.matrix_mode {
          MatrixMode::PositionAndVector | MatrixMode::Position => {
            self.position_vector_sp = (self.position_vector_sp as i8).wrapping_sub(offset).clamp(0, 63) as u8;

            if self.position_vector_sp > 30 {
              self.gxstat.matrix_stack_error = true;
            }

            self.current_position_matrix = self.position_stack[(self.position_vector_sp as usize) & 31];
            self.current_vector_matrix = self.vector_stack[(self.position_vector_sp as usize) & 31];

            self.clip_mtx_recalculate = true;
          }
          MatrixMode::Projection => {
            if self.projection_sp > 0 {
              self.projection_sp -= 1;
            }

            self.current_projection_matrix = self.projection_stack;

            self.clip_mtx_recalculate = true;
          }
          MatrixMode::Texture => {
            if self.texture_sp > 0 {
              self.texture_sp -= 1;
            }

            self.current_texture_matrix = self.texture_stack;
          }
        }
      }
      Shininess => {
        if !self.command_started {
          self.command_started = true;
          self.command_params = Shininess.get_num_params();
        }

        if self.command_params > 0 {
          let i = (Shininess.get_num_params() - self.command_params) * 4;

          self.shininess_table[i] = entry.param as u8;
          self.shininess_table[i + 1] = (entry.param >> 8) as u8;
          self.shininess_table[i + 2] = (entry.param >> 16) as u8;
          self.shininess_table[i + 3] = (entry.param >> 24) as u8;

          self.command_params -= 1;

          if self.command_params == 0 {
            self.command_started = false;
          }
        }
      }
      PolygonAttr => self.polygon_attributes = PolygonAttributes::from_bits_retain(entry.param),
      TexImageParam => self.texture_params.write(entry.param),
      PlttBase => self.palette_base = (entry.param & 0x1fff) << 4,
      SwapBuffers => {
        self.transluscent_polygon_sort = entry.param & 0b1 == 1;
        self.depth_buffering_with_w = entry.param >> 1 & 0b1 == 1;

        self.gxstat.geometry_engine_busy = true;

        self.polygons_ready = true;
      }
      Viewport => self.viewport.write(entry.param),
      MtxLoad4x4 => self.load_4_by_n_matrix(entry, MtxLoad4x4.get_num_params(), 4),
      MtxLoad4x3 => self.load_4_by_n_matrix(entry, MtxLoad4x3.get_num_params(), 3),
      DifAmb => {
        self.diffuse_reflection.write(entry.param as u16);

        if self.diffuse_reflection.set_vertex_color {
          self.vertex_color.write(entry.param as u16);
          self.vertex_color.to_rgb6();
        }

        self.ambient_reflection.write((entry.param >> 16) as u16);
      }
      SpeEmi => {
        self.specular_reflection.write(entry.param as u16);
        self.emission.write((entry.param >> 16) as u16);
      }
      LightColor => {
        let i = ((entry.param >> 30) & 0x3) as usize;

        self.lights[i].color.write(entry.param as u16);
      }
      LightVector => {
        let x = (entry.param as i16) << 6 >> 6;
        let y = ((entry.param >> 4) as i16) >> 6;
        let z = ((entry.param >> 14) as i16) >> 6;
        let i = ((entry.param >> 30) & 0x3) as usize;

        let transformed = self.current_vector_matrix.multiply_row(&[x as i32, y as i32, z as i32, 0], 12);

        self.lights[i].x = transformed[0] as i16;
        self.lights[i].y = transformed[1] as i16;
        self.lights[i].z = transformed[2] as i16;

        self.lights[i].half_vector = [
          transformed[0] / 2,
          transformed[1] / 2,
          (transformed[2] - 0x200) / 2
        ];
      }
      Color => {
        self.vertex_color.write(entry.param as u16);
        self.vertex_color.to_rgb6();
      }
      VecTest => {
        let x = (entry.param as i16) << 6 >> 6;
        let y = ((entry.param >> 4) as i16) >> 6;
        let z = ((entry.param >> 14) as i16) >> 6;

        let transformed = self.current_vector_matrix.multiply_row(&[x as i32, y as i32, z as i32, 0], 12);

        self.vec_result = transformed;
      }
      BeginVtxs => {
        self.primitive_type = PrimitiveType::from(entry.param & 0x3);

        self.swap_vertices = false;
        self.internal_polygon_attributes = self.polygon_attributes;

        self.max_vertices = self.primitive_type.get_num_vertices();

        self.current_vertices.clear();
      }
      MtxPush => {
        match self.matrix_mode {
          MatrixMode::PositionAndVector | MatrixMode::Position => {
            if self.position_vector_sp > 30 {
              self.gxstat.matrix_stack_error = true;
            }

            self.position_stack[(self.position_vector_sp & 31) as usize] = self.current_position_matrix;
            self.vector_stack[(self.position_vector_sp & 31) as usize] = self.current_vector_matrix;

            self.position_vector_sp += 1;
          }
          MatrixMode::Projection => {
            self.projection_stack = self.current_projection_matrix;

            self.projection_sp += 1;
          }
          MatrixMode::Texture => {
            self.texture_stack = self.current_texture_matrix;

            self.texture_sp += 1;
          }
        }
      }
      MtxTrans => {
        if !self.command_started {
          self.command_started = true;
          self.command_params = MtxTrans.get_num_params();
        }

        if self.command_params > 0 {
          let index = MtxTrans.get_num_params() - self.command_params;

          self.translation_vector[index] = entry.param as i32;

          self.command_params -= 1;

          if self.command_params == 0 {
            match self.matrix_mode {
              MatrixMode::Position => {
                self.current_position_matrix.translate(&self.translation_vector);

                self.clip_mtx_recalculate = true;
              }
              MatrixMode::PositionAndVector => {
                self.current_position_matrix.translate(&self.translation_vector);
                self.current_vector_matrix.translate(&self.translation_vector);

                self.clip_mtx_recalculate = true;
              }
              MatrixMode::Projection => {
                self.current_projection_matrix.translate(&self.translation_vector);
                self.clip_mtx_recalculate = true;
              }
              MatrixMode::Texture => {
                self.current_texture_matrix.translate(&self.translation_vector);
              }
            }

            self.command_started = false;
          }
        }
      }
      Texcoord => {
        self.texcoord.u = entry.param as i16;
        self.texcoord.v = (entry.param >> 16) as i16;

        self.original_texcoord = self.texcoord;

        if self.texture_params.transformation_mode == TransformationMode::TexCoord {
          let matrix = self.current_texture_matrix.0;

          let u = self.original_texcoord.u;
          let v = self.original_texcoord.v;

          self.texcoord.u = ((u as i64 * matrix[0][0] as i64 + v as i64 * matrix[1][0] as i64 + matrix[2][0] as i64 + matrix[3][0] as i64) >> 12) as i16;
          self.texcoord.v = ((u as i64 * matrix[0][1] as i64 + v as i64 * matrix[1][1] as i64 + matrix[2][1] as i64 + matrix[3][1] as i64) >> 12) as i16;
        }
      }
      Vtx16 => {
        if !self.command_started {
          self.command_started = true;

          self.command_params = Vtx16.get_num_params();
        }

        if self.command_params > 0 {
          self.command_params -= 1;

          if self.command_params == 1 {
            self.current_vertex.x = entry.param as i16;
            self.current_vertex.y = (entry.param >> 16) as i16;

          } else if self.command_params == 0 {
            self.current_vertex.z = entry.param as i16;

            self.add_vertex();

            self.command_started = false;
          }
        }
      }
      VtxXy => {
        self.current_vertex.x = entry.param as i16;
        self.current_vertex.y = (entry.param >> 16) as i16;

        self.add_vertex();
      }
      MtxScale => {
        if !self.command_started {
          self.command_started = true;
          self.scale_vector = [0; 3];
          self.command_params = MtxScale.get_num_params();
        }

        if self.command_params > 0 {
          let index = MtxScale.get_num_params() - self.command_params;

          self.scale_vector[index] = entry.param as i32;

          self.command_params -= 1;

          if self.command_params == 0 {
            self.command_started = false;

            match self.matrix_mode {
              MatrixMode::Position | MatrixMode::PositionAndVector => {
                self.current_position_matrix.scale(&self.scale_vector);

                self.clip_mtx_recalculate = true;
              }
              MatrixMode::Projection => {
                self.current_projection_matrix.scale(&self.scale_vector);
                self.clip_mtx_recalculate = true;
              }
              MatrixMode::Texture => {
                self.current_texture_matrix.scale(&self.scale_vector);
              }
            }
          }
        }
      }
      MtxMult4x3 => {
        self.multiply_m_by_n(4, 3, entry);
      }
      Vtx10 => {
        self.current_vertex.x = (entry.param as i16) << 6;
        self.current_vertex.y = ((entry.param >> 10) as i16) << 6;
        self.current_vertex.z = ((entry.param >> 20) as i16) << 6;

        self.add_vertex();
      }
      VtxXz => {
        self.current_vertex.x = entry.param as i16;
        self.current_vertex.z = (entry.param >> 16) as i16;

        self.add_vertex();
      }
      MtxStore => {
        let offset = entry.param & 0x1f;

        if offset > 30 {
          self.gxstat.matrix_stack_error = true;
        }

        match self.matrix_mode {
          MatrixMode::Position | MatrixMode::PositionAndVector => {
            self.position_stack[offset as usize] = self.current_position_matrix;
            self.vector_stack[offset as usize] = self.current_vector_matrix;

          }
          MatrixMode::Projection => {
            self.projection_stack = self.current_projection_matrix;
          }
          MatrixMode::Texture => {
            self.texture_stack = self.current_texture_matrix;
          }
        }
      }
      MtxMult4x4 => self.multiply_m_by_n(4, 4, entry),
      VtxYz => {
        self.current_vertex.y = entry.param as i16;
        self.current_vertex.z = (entry.param >> 16) as i16;

        self.add_vertex()
      }
      MtxMult3x3 => self.multiply_m_by_n(3, 3, entry),
      MtxRestore => {
        match self.matrix_mode {
          MatrixMode::Position | MatrixMode::PositionAndVector => {
            let offset = entry.param & 0x1f;

            if offset > 30 {
              self.gxstat.matrix_stack_error = true;
            }

            self.current_position_matrix = self.position_stack[offset as usize];
            self.current_vector_matrix = self.vector_stack[offset as usize];


            self.clip_mtx_recalculate = true;
          }
          MatrixMode::Projection => {
            self.current_projection_matrix = self.projection_stack;
            self.clip_mtx_recalculate = true;
          }
          MatrixMode::Texture => {
            self.current_texture_matrix = self.texture_stack;
          }
        }
      }
      Normal => {
        let x = (entry.param as i16) << 6 >> 6;
        let y = (entry.param >> 4) as i16 >> 6;
        let z = (entry.param >> 14) as i16 >> 6;

        let normal = [x as i32, y as i32, z as i32, 0];

        if self.texture_params.transformation_mode == TransformationMode::Normal {
          let transformed = self.current_texture_matrix.multiply_normal(&normal);

          self.texcoord.u = transformed[0] + self.original_texcoord.u;
          self.texcoord.v = transformed[1] + self.original_texcoord.v;
        }

        self.apply_lighting(&normal);
      }
      VtxDiff => {
        let x = (entry.param as i16) << 6 >> 6;
        let y = (entry.param >> 4) as i16 >> 6;
        let z = (entry.param >> 14) as i16 >> 6;

        self.current_vertex.x = self.current_vertex.x.wrapping_add(x);
        self.current_vertex.y = self.current_vertex.y.wrapping_add(y);
        self.current_vertex.z = self.current_vertex.z.wrapping_add(z);

        self.add_vertex();
      }
      BoxTest => {
        if !self.command_started {
          self.command_started = true;

          self.command_params = BoxTest.get_num_params();
          self.max_params = self.command_params;

          self.box_test = box_test::BoxTest::new();
        }

        let index = self.max_params - self.command_params;

        if index == 0 {
          self.box_test.x = entry.param as i16;
          self.box_test.y = (entry.param >> 16) as i16;
        } else if index == 1 {
          self.box_test.z = entry.param as i16;
          self.box_test.width = (entry.param >> 16) as i16;
        } else {
          self.box_test.height = entry.param as i16;
          self.box_test.depth = (entry.param >> 16) as i16;
        }

        self.command_params -= 1;

        if self.command_params == 0 {
          self.command_started = false;
          if self.clip_mtx_recalculate {
            self.recalculate_clip_matrix();
          }

          self.gxstat.box_test_result = self.box_test.do_test(self.clip_matrix);
        }
      }
      _ => panic!("command not iplemented yet: {:?}", entry.command)
    }
  }

  fn apply_lighting(&mut self, coordinates: &[i32]) {
    let normal = self.current_vector_matrix.multiply_row(&coordinates, 12);

    let mut color = [self.emission.r as i32, self.emission.g as i32, self.emission.b as i32];

    for (i, light) in self.lights.iter().enumerate() {
      if self.internal_polygon_attributes.light_enabled(i) {
        let light_direction = [light.x as i32, light.y as i32, light.z as i32, 0];

        let diffuse_level = ((-light_direction
          .iter()
          .zip(normal.iter())
          .fold(0_i32, |acc, (a, b)| {
              acc.wrapping_add((*a as i64 * *b as i64) as i32)
          }))
          >> 9)
          .max(0);

        let mut shininess_level = (-(light
          .half_vector
          .iter()
          .zip(normal.iter())
          .fold(0i32, |acc, (a, b)| acc.wrapping_add((*a as i64 * *b as i64) as i32))) >> 9)
          .max(0);

        if shininess_level >= 0x200 {
          shininess_level = (0x400i32.wrapping_sub(shininess_level)) & 0x1ff;
        }

        shininess_level = (((shininess_level * shininess_level) >> 9) - 0x100).max(0);

        if self.specular_reflection.shininess_table_enable {
          shininess_level = self.shininess_table[(shininess_level / 2) as usize] as i32;
        }

        color[0] += ((self.specular_reflection.r as i64 * light.color.r as i64 * shininess_level as i64) >> 13) as i32;
        color[0] += ((self.diffuse_reflection.r as i64 * light.color.r as i64 * diffuse_level as i64) >> 14) as i32;
        color[0] += ((self.ambient_reflection.r as i64 * light.color.r as i64) >> 5) as i32;

        color[1] += ((self.specular_reflection.g as i64 * light.color.g as i64 * shininess_level as i64) >> 13) as i32;
        color[1] += ((self.diffuse_reflection.g as i64 * light.color.g as i64 * diffuse_level as i64) >> 14) as i32;
        color[1] += ((self.ambient_reflection.g as i64 * light.color.g as i64) >> 5) as i32;

        color[2] += ((self.specular_reflection.b as i64 * light.color.b as i64 * shininess_level as i64) >> 13) as i32;
        color[2] += ((self.diffuse_reflection.b as i64 * light.color.b as i64 * diffuse_level as i64) >> 14) as i32;
        color[2] += ((self.ambient_reflection.b as i64 * light.color.b as i64) >> 5) as i32;
      }
    }

    for i in 0..color.len() {
      color[i] = color[i].clamp(0, 0x1f);
    }

    let mut color = Color {
      r: color[0] as u8,
      g: color[1] as u8,
      b: color[2] as u8,
      alpha: None
    };

    color.to_rgb6();

    self.vertex_color = color;
  }

  fn multiply_m_by_n(&mut self, m: usize, n: usize, entry: GeometryCommandEntry) {
    use Command::*;
    if !self.command_started {
      self.temp_matrix = Matrix::new();

      self.command_started = true;
      self.command_params = match (m, n) {
        (4, 4) => MtxMult4x4.get_num_params(),
        (4, 3) => MtxMult4x3.get_num_params(),
        (3, 3) => MtxMult3x3.get_num_params(),
        _ => panic!("invalid values given for multiply m x n: {m} x {n}")
      };

      self.max_params = self.command_params
    }

    if self.command_params > 0 {
      let index = self.max_params - self.command_params;
      let row = index / n;
      let column = index % n;

      self.temp_matrix.0[row][column as usize] = entry.param as i32;

      self.command_params -= 1;

      if self.command_params == 0 {
        let matrices = match self.matrix_mode {
          MatrixMode::Position => {
            self.clip_mtx_recalculate = true;

            [Some(&mut self.current_position_matrix), None]
          }
          MatrixMode::PositionAndVector => {
            self.clip_mtx_recalculate = true;

            [Some(&mut self.current_position_matrix), Some(&mut self.current_vector_matrix)]
          }
          MatrixMode::Projection => {
            self.clip_mtx_recalculate = true;

            [Some(&mut self.current_projection_matrix), None]
          }
          MatrixMode::Texture => {
           [Some(&mut self.current_texture_matrix), None]
          }
        };

        for matrix in matrices {
          if let Some(matrix) = matrix {
            match (m, n) {
              (4, 4) => *matrix = self.temp_matrix * *matrix,
              (4, 3) => matrix.multiply_4x3(self.temp_matrix),
              (3, 3) => matrix.multiply_3x3(self.temp_matrix),
              _ => panic!("invalid option given for m x n: {m} x {n}")
            }
          }
        }

        self.command_started = false;
      }
    }
  }

  fn add_vertex(&mut self) {
    let vertex = self.current_vertex;

    // TODO: check polygon ram overflow here
    if self.polygon_buffer.len() == POLYGON_BUFFER_SIZE {
      return;
    }

    // recalculate clip matrix
    if self.clip_mtx_recalculate {
      self.recalculate_clip_matrix();
    }

    self.current_vertex.transformed = self.clip_matrix.multiply_row(&[vertex.x as i32, vertex.y as i32, vertex.z as i32, 0x1000], 12);

    if self.texture_params.transformation_mode == TransformationMode::Vertex {
      let transformed = self.current_texture_matrix.multiply_row(&[vertex.x as i32, vertex.y as i32, vertex.z as i32, 0], 24);

      self.texcoord.u = transformed[0] as i16 + self.original_texcoord.u;
      self.texcoord.v = transformed[1] as i16 + self.original_texcoord.v;
    }

    self.current_vertex.texcoord = self.texcoord;
    self.current_vertex.color = self.vertex_color;

    self.current_vertices.push(self.current_vertex);
    if self.current_vertices.len() == self.max_vertices {
      match self.primitive_type {
        PrimitiveType::QuadStrips => {
          let new_vertex0 = self.current_vertices[2];
          let new_vertex1 = self.current_vertices[3];
          self.current_vertices.swap(2, 3);
          self.submit_polygon();
          self.current_vertices.push(new_vertex0);
          self.current_vertices.push(new_vertex1);
        }
        PrimitiveType::TriangleStrips => {
          let new_vertex0 = self.current_vertices[1];
          let new_vertex1 = self.current_vertices[2];
          if self.swap_vertices {
            self.current_vertices.swap(1, 2);
          }
          self.swap_vertices = !self.swap_vertices;
          self.submit_polygon();
          self.current_vertices.push(new_vertex0);
          self.current_vertices.push(new_vertex1);
        }
        _ => self.submit_polygon()
      }
    }
  }

  fn submit_polygon(&mut self) {
    let a = (
      self.current_vertices[0].transformed[0] - self.current_vertices[1].transformed[0],
      self.current_vertices[0].transformed[1] - self.current_vertices[1].transformed[1],
      self.current_vertices[0].transformed[3] - self.current_vertices[1].transformed[3]
    );

    let b= (
      self.current_vertices[2].transformed[0] - self.current_vertices[1].transformed[0],
      self.current_vertices[2].transformed[1] - self.current_vertices[1].transformed[1],
      self.current_vertices[2].transformed[3] - self.current_vertices[1].transformed[3]
    );

    let mut normal = [
      (a.1 as i64 * b.2 as i64) - (a.2 as i64 * b.1 as i64),
      (a.2 as i64 * b.0 as i64) - (a.0 as i64 * b.2 as i64),
      (a.0 as i64 * b.1 as i64) - (a.1 as i64 * b.0 as i64)
    ];

    while (normal[0] >> 31) ^ (normal[0] >> 63) != 0 ||
      (normal[1] >> 31) ^ (normal[1] >> 63) != 0 ||
      (normal[2] >> 31) ^ (normal[2] >> 63) != 0 {
        normal[0] >>= 4;
        normal[1] >>= 4;
        normal[2] >>= 4;
    }

    let transformed = self.current_vertices[0].transformed;

    let dot_product = normal[0] * transformed[0] as i64 + normal[1] * transformed[1] as i64 + normal[2] * transformed[3] as i64;

    if dot_product == 0 {
      self.current_vertices.clear();
      return;
    }

    let mut is_front = false;

    if dot_product < 0 {
      is_front = true;

      if !self.internal_polygon_attributes.contains(PolygonAttributes::SHOW_FRONT_SURFACE) {
        self.current_vertices.clear();
        return;
      }
    } else if dot_product > 0 {
      if !self.internal_polygon_attributes.contains(PolygonAttributes::SHOW_BACK_SURFACE) {
        self.current_vertices.clear();
        return;
      }
    }

    for i in (0..3).rev() {
      Self::clip_plane(i, &mut self.current_vertices);
    }

    if self.current_vertices.is_empty() {
      return;
    }

    let mut polygon = Polygon {
      palette_base: self.palette_base as usize,
      start: self.vertices_buffer.len(),
      end: self.vertices_buffer.len() + self.current_vertices.len(),
      attributes: self.internal_polygon_attributes,
      is_front,
      tex_params: self.texture_params,
      top: 0,
      bottom: 191,
      primitive_type: self.primitive_type
    };

    let mut size = 0;
    for vertex in self.current_vertices.iter() {
      let w = vertex.transformed[3] as u32;
      // need to specify size < 32 for release builds since the number
      // *will* wrap around on a release build
      while w >> size != 0 && size < 32 {
        size += 4;
      }
    }
    let (mut top, mut bottom) = (191, 0);

    for vertex in self.current_vertices.drain(..) {
      let mut temp = vertex.clone();

      let transformed = temp.transformed;
      // per martin korth:
      // screen_x = (xx+ww)*viewport_width / (2*ww) + viewport_x1
      // screen_y = (yy+ww)*viewport_height / (2*ww) + viewport_y1

      temp.screen_x = if transformed[3] == 0 {
        0
      } else {
        let mut w = transformed[3];
        let mut x_offset = transformed[0] + w;

        if w > 0xffff {
          x_offset >>= 1;
          w >>= 1;
        }

        let denominator = 2 * w;
        (x_offset * self.viewport.width() / denominator + self.viewport.x1 as i32) as u32
      };

      temp.screen_y = if transformed[3] == 0 {
        0
      } else {
        let mut w = transformed[3];
        let mut y_offset = -transformed[1] + w;

        if w > 0xffff {
          w >>= 1;
          y_offset >>= 1;
        }

        let denominator = 2 * w;
        (y_offset * self.viewport.height() / denominator + self.viewport.y1 as i32) as u32
      };

      let w = if transformed[3] > 0 {
        transformed[3]
      } else {
        1
      };

      temp.z_depth = ((((transformed[2] as i64 * 0x4000 / w as i64) + 0x3fff) * 0x200) & 0xffffff) as u32;
      temp.normalized_w = if size < 16 {
        transformed[3] << (16 - size)
      } else {
        transformed[3] >> (size - 16)
      } as i16;

      if vertex.screen_y < top {
        top = vertex.screen_y;
      }
      if vertex.screen_y > bottom {
        bottom = vertex.screen_y;
      }
      self.vertices_buffer.push(temp);
    }

    polygon.top = top;
    polygon.bottom = bottom;

    self.polygon_buffer.push(polygon);
  }

  fn clip_plane(index: usize, vertices: &mut Vec<Vertex>) {
    let mut temp: Vec<Vertex> = Vec::with_capacity(10);

    for i in 0..vertices.len() {
      let current = vertices[i];
      let previous_index = if i == 0 {
        vertices.len() - 1
      } else {
        i - 1
      };

      let previous = vertices[previous_index];

      // current is inside the positive part of plane
      if current.transformed[index] <= current.transformed[3] {

        // previous point is outside
        if previous.transformed[index] > previous.transformed[3] {
          temp.push(Self::find_plane_intersection(index, current, previous, true));
        }
        temp.push(current.clone());

      } else if previous.transformed[index] <= previous.transformed[3] {
        temp.push(Self::find_plane_intersection(index, previous, current, true));
      }
    }

    vertices.clear();

    for i in 0..temp.len() {
      let current = temp[i];
      let previous_i = if i == 0 { temp.len() - 1} else { i - 1 };

      let previous = temp[previous_i];

      // current is inside negative part of plane
      if current.transformed[index] >= -current.transformed[3] {
        if previous.transformed[index] < -previous.transformed[3] {
          // previous is outside negative part of plane
          let vertex = Self::find_plane_intersection(index, current, previous, false);
          vertices.push(vertex);
        }
        vertices.push(current.clone());
      } else if previous.transformed[index] >= -previous.transformed[3] {

        let vertex = Self::find_plane_intersection(index, previous, current, false);
        vertices.push(vertex);
      }
    }

  }

  fn find_plane_intersection(index: usize, inside: Vertex, outside: Vertex, positive_plane: bool) -> Vertex {
    let sign = if positive_plane { 1 } else { -1 };

    let numerator = inside.transformed[3] as i64 - sign * inside.transformed[index] as i64;
    let denominator = numerator as i64 - (outside.transformed[3] as i64 - sign * outside.transformed[index] as i64);

    let new_w = Self::calculate_coordinates(
      index,
      3,
      inside,
      outside,
      numerator,
      denominator,
      sign,
      0
    );

    let x = Self::calculate_coordinates(index, 0, inside, outside, numerator, denominator, sign, new_w) as i32;
    let y = Self::calculate_coordinates(index, 1, inside, outside, numerator, denominator, sign, new_w) as i32;
    let z = Self::calculate_coordinates(index, 2, inside, outside, numerator, denominator, sign, new_w) as i32;

    let r = Self::interpolate(inside.color.r as i64, outside.color.r as i64, numerator, denominator) as u8;
    let g = Self::interpolate(inside.color.g as i64, outside.color.g as i64, numerator, denominator) as u8;
    let b = Self::interpolate(inside.color.b as i64, outside.color.b as i64, numerator, denominator) as u8;

    let mut texcoord = Texcoord::new();

    texcoord.u = Self::interpolate(inside.texcoord.u as i64, outside.texcoord.u as i64, numerator, denominator) as i16;
    texcoord.v = Self::interpolate(inside.texcoord.v as i64, outside.texcoord.v as i64, numerator, denominator) as i16;

    Vertex {
      transformed: [x, y, z, new_w as i32],
      screen_x: 0,
      screen_y: 0,
      z_depth: 0,
      x: 0, // these don't matter after this point, todo: maybe merge transformed and these together
      y: 0, // see above
      z: 0, // see above
      texcoord,
      color: Color {
        r,
        g,
        b,
        alpha: None
      },
      normalized_w: 0
    }
  }

  fn interpolate(inside: i64, outside: i64, numerator: i64, denominator: i64) -> i64 {
    inside + (outside - inside) * numerator / denominator
  }

  fn calculate_coordinates(
    current_index: usize,
    index: usize,
    inside: Vertex,
    outside: Vertex,
    numerator: i64,
    denominator: i64,
    sign: i64,
    w: i64) -> i64
  {
    if current_index == index {
      sign * w as i64
    } else {
      Self::interpolate(inside.transformed[index] as i64, outside.transformed[index] as i64, numerator, denominator)
    }
  }

  fn recalculate_clip_matrix(&mut self) {
    self.clip_matrix = self.current_position_matrix * self.current_projection_matrix;

    self.clip_mtx_recalculate = false;
  }

  fn load_4_by_n_matrix(&mut self, entry: GeometryCommandEntry, num_params: usize, n: usize) {
    if !self.command_started {
      self.command_started = true;
      self.command_params = num_params;

      self.temp_matrix = Matrix::new();
    }

    if self.command_params > 0 {

      let index_raw = num_params - self.command_params;

      let row = index_raw / n;
      let column = index_raw % n;

      self.temp_matrix.0[row][column] = entry.param as i32;

      self.command_params -= 1;

      if self.command_params == 0 {
        self.load_matrix();

        self.command_started = false;
      }
    }
  }

  fn load_matrix(&mut self) {
    match self.matrix_mode {
      MatrixMode::Position  => {
        self.current_position_matrix = self.temp_matrix;

        self.clip_mtx_recalculate = true;
      }
      MatrixMode::Projection => {
        self.current_projection_matrix = self.temp_matrix;
        self.clip_mtx_recalculate = true;
      }
      MatrixMode::Texture => {
        self.current_texture_matrix = self.temp_matrix;
      }
      MatrixMode::PositionAndVector => {
        self.current_position_matrix = self.temp_matrix;
        self.current_vector_matrix = self.temp_matrix;

        self.clip_mtx_recalculate = true;
      }
    }
  }

  fn process_commands(&mut self, value: u32, interrupt_request: &mut InterruptRequestRegister) {
    while self.packed_commands != 0 {
      let current_command = self.current_command;

      if current_command != Command::Nop {
        self.push_command(GeometryCommandEntry::from(current_command, value), interrupt_request);
      }

      if self.params_processed == self.num_params {
        self.packed_commands >>= 8;
        if self.packed_commands != 0 {
          self.current_command = Command::from(self.packed_commands as u8);
          self.num_params = self.current_command.get_num_params();
          self.params_processed = 0;

          if self.num_params > 0 {
            break;
          }
        }
      } else {
        break;
      }
    }
  }

  pub fn push_command(&mut self, entry: GeometryCommandEntry, interrupt_request: &mut InterruptRequestRegister) {
    self.fifo.push_back(entry);

    self.execute_commands(interrupt_request);
  }

  pub fn write_geometry_fifo(&mut self, value: u32, interrupt_request: &mut InterruptRequestRegister) {
    if self.packed_commands == 0 {
      if value == 0 {
        // there's nothing to do here, just short circuit early
        return;
      }

      self.packed_commands = value;

      let current_command = Command::from(self.packed_commands as u8);

      self.num_params = current_command.get_num_params();
      self.params_processed = 0;

      self.current_command = current_command;

      if self.num_params > 0 {
        return;
      }
    } else {
      self.params_processed += 1;
    }

    self.process_commands(value, interrupt_request);

  }
}