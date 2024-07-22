use super::date_time_register::DateTimeRegister;

#[derive(Copy, Clone, PartialEq, Debug)]
enum CommandMode {
  AwaitingCommand(bool),
  AcceptingCommand,
  ExecutingCommand,
  FinishingCommand
}

#[derive(Copy, Clone, PartialEq)]
enum Access {
  Reading = 0,
  Writing = 1,
  None
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum Param {
  StatusRegister1 = 0,
  StatusRegister2 = 1,
  DateTime = 2,
  Time = 3,
  AlarmTime1FrequencyDuty = 4,
  AlarmTime2 = 5,
  ClockAdjust = 6,
  None
}

impl Param {
  pub fn from(val: u8) -> Self {
    match val {
      0 => Param::StatusRegister1,
      1 => Param::StatusRegister2,
      2 => Param::DateTime,
      3 => Param::Time,
      4 => Param::AlarmTime1FrequencyDuty,
      5 => Param::AlarmTime2,
      6 => Param::ClockAdjust,
      _ => panic!("invalid value given for param: {val}")
    }
  }
}

pub struct RealTimeClockRegister {
  data: bool,
  sck: bool,
  cs: bool,
  data_direction: bool,
  sck_direction: bool,
  cs_direction: bool,
  current_command_byte: u8,
  current_command_bits: usize,
  mode: CommandMode,
  access: Access,
  param: Param,
  current_data_byte: u8,
  date_time: DateTimeRegister,
  data_bytes_remaining: u8,
  current_data_bits: usize
}

impl RealTimeClockRegister {
  pub fn new() -> Self {
    Self {
      data: false,
      sck: false,
      cs: false,
      data_direction: false,
      sck_direction: false,
      cs_direction: false,
      mode: CommandMode::AwaitingCommand(false),
      current_command_byte: 0,
      current_command_bits: 0,
      param: Param::None,
      access: Access::None,
      current_data_byte: 0,
      data_bytes_remaining: 0,
      date_time: DateTimeRegister::new(),
      current_data_bits: 0
    }
  }

  pub fn write(&mut self, val: u16) {
    let previous_sck = self.sck;

    self.data_direction = (val >> 4) & 0b1 == 1;
    self.sck_direction = (val >> 5) & 0b1 == 1;
    self.cs_direction = (val >> 6) & 0b1 == 1;

    if self.data_direction {
      self.data = val & 0b1 == 1;
    }
    if self.sck_direction {
      self.sck = (val >> 1) & 0b1 == 1;
    }
    if self.cs_direction {
      self.cs = (val >> 2) & 0b1 == 1;
    }

    self.on_write(previous_sck);
  }

  fn on_write(&mut self, previous_sck: bool) {
    match self.mode {
      CommandMode::AwaitingCommand(false) => {
        if !self.cs {
          self.mode = CommandMode::AwaitingCommand(true)
        }
      }
      CommandMode::AwaitingCommand(true) => {
        if self.cs && self.sck {
          self.mode = CommandMode::AcceptingCommand;
          self.current_command_byte = 0;
          self.current_command_bits = 0;
        }
      }
      CommandMode::AcceptingCommand => {
        if previous_sck && !self.sck {
          if self.current_command_bits < 8 {
            self.current_command_byte = (self.current_command_byte << 1) | self.data as u8;
            self.current_command_bits += 1;

            if self.current_command_bits == 8 {
              self.param = Param::from((self.current_command_byte >> 1) & 0x7);

              self.data_bytes_remaining = match self.param {
                Param::AlarmTime1FrequencyDuty | Param::AlarmTime2 | Param::Time => 3,
                Param::DateTime => 7,
                _ => 1
              };

              self.current_data_byte = 0;
              self.current_command_bits = 0;

              if self.current_command_byte & 0b1 == 1 {
                self.read_data();
                self.access = Access::Reading;
              } else {
                self.access = Access::Writing;
              }

              self.current_data_bits = 0;
              self.current_command_byte = 0;

              self.mode = CommandMode::ExecutingCommand;
            }
          }
        }
      }
      CommandMode::ExecutingCommand => {
        if previous_sck && !self.sck {
          match self.access {
            Access::Reading => {
              if self.current_data_bits < 8 {
                self.data = self.current_data_byte & 0b1 == 1;

                self.current_data_byte >>= 1;

                self.current_data_bits += 1;

                if self.current_data_bits == 8 {
                  self.on_finished_transmitting();
                  self.current_data_bits = 0;
                }
              }
            }
            Access::Writing => {
              if self.current_data_bits < 8 {
                self.current_data_byte = self.current_data_byte | (self.data as u8) << self.current_data_bits;
                self.current_data_bits += 1;

                if self.current_data_bits == 8 {
                  self.write_data();
                  self.on_finished_transmitting();
                  self.current_data_bits = 0;
                }
              }
            }
            _ => unreachable!()
          }
        }
      }
      CommandMode::FinishingCommand => {
        if !self.cs {
          self.mode = CommandMode::AwaitingCommand(false);
        }
      }
    }
  }

  fn on_finished_transmitting(&mut self) {
    if self.data_bytes_remaining == 0 {
      self.mode = CommandMode::FinishingCommand;
    } else {
      self.current_data_bits = 0;
      self.current_data_byte = 0;

      if self.access == Access::Reading {
        self.read_data();
      }

      self.mode = CommandMode::ExecutingCommand;
    }
  }

  fn read_data(&mut self) {
    self.data_bytes_remaining -= 1;

    self.current_data_byte = match self.param {
      Param::StatusRegister1 => self.date_time.read_status1(),
      Param::StatusRegister2 => self.date_time.read_status2(),
      Param::AlarmTime1FrequencyDuty if self.date_time.status_register2.int1_mode & 0b1 == 0 => {
        self.date_time.frequency_duty_setting as u8
      }
      Param::AlarmTime1FrequencyDuty => self.date_time.read_alarm1(2 - self.data_bytes_remaining),
      Param::AlarmTime2 => self.date_time.read_alarm2(2 - self.data_bytes_remaining),
      Param::DateTime => self.date_time.read(6 - self.data_bytes_remaining),
      Param::Time => self.date_time.read_time(2 - self.data_bytes_remaining),
      Param::ClockAdjust => self.date_time.clock_adjust,
      Param::None => unreachable!()
    }
  }

  fn write_data(&mut self) {
    self.data_bytes_remaining -= 1;

    match self.param {
      Param::StatusRegister1 => self.date_time.write_status1(self.current_data_byte),
      Param::StatusRegister2 => self.date_time.write_status2(self.current_data_byte),
      Param::AlarmTime1FrequencyDuty if self.date_time.status_register2.int1_mode & 0b1 == 0 => {
        self.date_time.frequency_duty_setting = self.current_data_byte != 0
      }
      Param::AlarmTime1FrequencyDuty => self.date_time.write_alarm1(self.current_data_byte, 2 - self.data_bytes_remaining),
      Param::AlarmTime2 => self.date_time.write_alarm2(self.current_data_byte, 2 - self.data_bytes_remaining),
      Param::DateTime => self.date_time.write(self.current_data_byte, 6 - self.data_bytes_remaining),
      Param::Time => self.date_time.write_time(self.current_data_byte, 2 - self.data_bytes_remaining),
      Param::ClockAdjust => self.date_time.clock_adjust = self.current_data_byte,
      Param::None => unreachable!()
    }
  }

  pub fn read(&self) -> u8 {
    let data = if !self.data_direction { self.data } else { false };
    let sck = if !self.sck_direction { self.sck } else { false };
    let cs = if !self.cs_direction { self.cs } else { false };

    data as u8 |
      (sck as u8) << 1 |
      (cs as u8) << 2 |
      (self.data_direction as u8) << 4 |
      (self.sck_direction as u8) << 5 |
      (self.cs_direction as u8) << 6
  }
}