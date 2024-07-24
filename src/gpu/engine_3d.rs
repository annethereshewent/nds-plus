use std::collections::VecDeque;

use super::registers::geometry_status_register::GeometryStatusRegister;

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
pub struct GeometryCommand {
  command: Command,
  param: u32
}

impl GeometryCommand {
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
  fifo: VecDeque<GeometryCommand>,
  sent_commands: bool,
  packed_commands: VecDeque<u8>,
  command_params: Vec<u32>,
  current_command: Option<Command>,
  params_processed: usize,
  num_params: usize,
  gxstat: GeometryStatusRegister
}

impl Engine3d {
  pub fn new() -> Self {
    Self {
      fifo: VecDeque::with_capacity(256),
      sent_commands: false,
      packed_commands: VecDeque::with_capacity(4),
      command_params: Vec::new(),
      current_command: None,
      params_processed: 0,
      num_params: 0,
      gxstat: GeometryStatusRegister::new()
    }
  }

  pub fn read_geometry_status(&self) -> u32 {
    self.gxstat.read(0, 0, &self.fifo)
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
        if let Some(command) = self.packed_commands.pop_front() {
          self.current_command = Some(Command::from(command));

          let mut current_command = self.current_command.unwrap();

          while current_command.get_num_params() == 0 {
            if current_command != Command::Nop {
              self.fifo.push_back(GeometryCommand::from(current_command, 0));
            }
            if let Some(command) = self.packed_commands.pop_front() {
              current_command = Command::from(command);
            } else {
              // we have finished processing all the packed commands
              self.sent_commands = false;
              break;
            }
          }

          self.params_processed = 1;

          self.num_params = current_command.get_num_params();

          if current_command != Command::Nop {
            self.fifo.push_back(GeometryCommand::from(current_command, value));
          }

          if self.params_processed == self.num_params {
            self.current_command = None;
          }
        } else {
          // we have finished processing all the packed commands
          self.sent_commands = false;
        }
      } else if self.params_processed < self.num_params {
        let current_command = self.current_command.unwrap();

        self.fifo.push_back(GeometryCommand::from(current_command, value));

        self.params_processed += 1;
      } else {
        self.current_command = None;
      }

    }
  }
}