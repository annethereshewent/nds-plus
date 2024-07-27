use std::collections::VecDeque;

use matrix::{Matrix, UNIT_MATRIX};

use super::{color::Color, registers::{clear_color_register::ClearColorRegister, fog_color_register::FogColorRegister, geometry_status_register::GeometryStatusRegister}};

pub  mod matrix;

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
  sent_commands: bool,
  packed_commands: VecDeque<u8>,
  current_command: Option<Command>,
  params_processed: usize,
  num_params: usize,
  gxstat: GeometryStatusRegister,
  clear_color: ClearColorRegister,
  clear_depth: u16,
  clear_offset_x: u16,
  clear_offset_y: u16,
  fog_color: FogColorRegister,
  fog_offset: u16,
  fog_table: [u8; 32],
  edge_colors: [Color; 8],
  toon_table: [Color; 32],
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
  projection_stack: Matrix
}

impl Engine3d {
  pub fn new() -> Self {
    Self {
      fifo: VecDeque::with_capacity(256),
      sent_commands: false,
      packed_commands: VecDeque::with_capacity(4),
      current_command: None,
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
      texture_sp: 0
    }
  }

  pub fn read_geometry_status(&self) -> u32 {
    self.gxstat.read(0, 0, &self.fifo)
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
    self.clear_depth = value & 0x7fff;
  }

  pub fn write_clear_image_offset(&mut self, value: u16) {
    self.clear_offset_x = value & 0xff;
    self.clear_offset_y = (value >> 8) & 0xff;
  }

  pub fn write_geometry_status(&mut self, value: u32) {
    self.gxstat.write(value);
  }

  pub fn write_geometry_command(&mut self, address: u32, value: u32) {
    let command = Command::from_address(address & 0xfff);

    self.fifo.push_back(GeometryCommandEntry::from(command, value));
  }

  pub fn start_rendering(&mut self) {
    while let Some(entry) = self.fifo.pop_front() {
      self.execute_command(entry);
    }
  }

  fn execute_command(&mut self, entry: GeometryCommandEntry) {
    use Command::*;
    match entry.command {
      EndVtxs => (), // just a NOP,
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
          }
          MatrixMode::PositionAndVector => {
            self.current_position_matrix = Matrix::new();
            self.current_vector_matrix = Matrix::new();
          }
          MatrixMode::Projection => {
            self.current_projection_matrix = Matrix::new();
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

            // TODO: set overflow true on value greater than or equal to 31

            self.current_position_matrix = self.position_stack[(self.position_vector_sp as usize) & 31].clone();
            self.current_vector_matrix = self.vector_stack[(self.position_vector_sp as usize) & 31].clone();
            // TODO: recalculate clip matrix
          }
          MatrixMode::Projection => {
            if self.projection_sp > 0 {
              self.projection_sp -= 1;
            }

            self.current_projection_matrix = self.projection_stack.clone();

            // TODO: recalculate clip matrix
          }
          MatrixMode::Texture => {
            if self.texture_sp > 0 {
              self.texture_sp -= 1;
            }

            self.current_texture_matrix = self.texture_stack.clone();
          }
        }
      }
      _ => panic!("command not implemented yet: {:?}", entry.command)
    }
  }

  fn process_commands(&mut self, value: u32) {
    while let Some(command) = self.packed_commands.pop_front() {
      self.current_command = Some(Command::from(command));

      let current_command = self.current_command.unwrap();

      self.params_processed = 1;

      self.num_params = current_command.get_num_params();

      if current_command != Command::Nop {
        self.fifo.push_back(GeometryCommandEntry::from(current_command, value));
      }

      if current_command.get_num_params() > 1 {
        break;
      }
    }

    if self.num_params == self.params_processed {
      self.sent_commands = false;
    }
  }

  pub fn write_geometry_fifo(&mut self, value: u32) {
    if !self.sent_commands {
      if value == 0 {
        // there's nothing to do here, just short circuit early
        return;
      }

      self.packed_commands = VecDeque::with_capacity(4);

      let mut val = value;

      while val != 0 {
        self.packed_commands.push_back(val as u8);
        val >>= 8;
      }

      self.sent_commands = true;
    } else {
      // process parameters for the commands
      if self.current_command.is_none() {
        self.process_commands(value);
      } else if self.params_processed < self.num_params {
        let current_command = self.current_command.unwrap();

        self.fifo.push_back(GeometryCommandEntry::from(current_command, value));

        self.params_processed += 1;

        if self.params_processed == self.num_params && self.packed_commands.is_empty() {
          self.sent_commands = false;
        }
      } else {
        self.process_commands(value);
      }
    }
  }
}