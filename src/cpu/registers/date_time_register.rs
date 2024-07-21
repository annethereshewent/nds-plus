use chrono::{Datelike, Local, Timelike};



pub struct DateTimeRegister {
  pub status_register1: StatusRegister1,
  pub status_register2: StatusRegister2,
  pub alarm1: AlarmRegister,
  pub alarm2: AlarmRegister,
  pub frequency_duty_setting: bool,
  pub clock_adjust: u8
}

impl DateTimeRegister {
  pub fn new() -> Self {
    Self {
      status_register1: StatusRegister1::new(),
      status_register2: StatusRegister2::new(),
      frequency_duty_setting: false,
      alarm1: AlarmRegister::new(),
      alarm2: AlarmRegister::new(),
      clock_adjust: 0
    }
  }

  fn to_bcd(value: u8) -> u8 {
    let tens = value / 10;
    let ones = value % 10;

    tens << 4 | ones
  }

  pub fn read_status1(&self) -> u8 {
    self.status_register1.read()
  }

  pub fn read_status2(&self) -> u8 {
    self.status_register2.read()
  }

  pub fn read_alarm1(&self, byte: u8) -> u8 {
    self.alarm1.read(byte)
  }
  pub fn read_alarm2(&self, byte: u8) -> u8 {
    self.alarm2.read(byte)
  }

  pub fn read(&self, byte: u8) -> u8 {
    let time = Local::now();

    let mut am_or_pm = false;

    let value = match byte {
      0 => time.year() as u32 - 2000,
      1 => time.month(),
      2 => time.day(),
      3 => time.weekday().num_days_from_monday(),
      4 => {
        let (am_pm, hour) = if self.status_register1.twenty_four_hour_mode {
          (time.hour() >= 12, time.hour())
        } else {
          time.hour12()
        };

        am_or_pm = am_pm;

        hour
      }
      5 => time.minute(),
      6 => time.second(),
      _ => unreachable!()
    };

    (am_or_pm as u8) << 6 | Self::to_bcd(value as u8)
  }

  pub fn read_time(&self, byte: u8) -> u8 {
    self.read(byte + 4)
  }

  pub fn write(&mut self, _value: u8, _byte: u8) {
    println!("Warning: ignoring setting the date");
  }

  pub fn write_time(&mut self, _value: u8, _byte: u8) {
    println!("Warning: ignoring setting the time");
  }

  pub fn write_status1(&mut self, value: u8) {
    self.status_register1.write(value);
  }

  pub fn write_status2(&mut self, value: u8) {
    self.status_register2.write(value);
  }

  pub fn write_alarm1(&mut self, value: u8, byte: u8) {
    self.alarm1.write(value, byte);
  }

  pub fn write_alarm2(&mut self, value: u8, byte: u8) {
    self.alarm2.write(value, byte);
  }
}

pub struct StatusRegister1 {
  general_purpose_bits: u8,
  twenty_four_hour_mode: bool
}

impl StatusRegister1 {
  pub fn new() -> Self {
    Self {
      general_purpose_bits: 0,
      twenty_four_hour_mode: false
    }
  }

  pub fn read(&self) -> u8 {
    (self.twenty_four_hour_mode as u8) << 1 |
      (self.general_purpose_bits) << 2
  }

  pub fn write(&mut self, value: u8) {
    self.twenty_four_hour_mode = (value >> 1) & 0b1 == 1;
    self.general_purpose_bits = (value >> 2) & 0x3;
  }
}

pub struct StatusRegister2 {
  pub int1_mode: u8,
  general_purpose_bits: u8,
  int2_enable: bool,
  test_mode: bool
}

impl StatusRegister2 {
  pub fn new() -> Self {
    Self {
      int1_mode: 0,
      general_purpose_bits: 0,
      int2_enable: false,
      test_mode: false
    }
  }

  pub fn read(&self) -> u8 {
    self.int1_mode |
      self.general_purpose_bits << 4 |
      (self.int2_enable as u8) << 6 |
      (self.test_mode as u8) << 7
  }

  pub fn write(&mut self, value: u8) {
    self.int1_mode = value & 0xf;
    self.general_purpose_bits = (value >> 4) & 0x3;
    self.int2_enable = (value >> 6) & 0b1 == 1;
    self.test_mode = (value >> 7) & 0b1 == 1;
  }
}

pub struct AlarmRegister {
  day: u8,
  cmp_day: bool,
  hour: u8,
  cmp_hour: bool,
  am_pm: bool,
  min: u8,
  cmp_min: bool
}

impl AlarmRegister {
  pub fn new() -> Self {
    Self {
      day: 0,
      cmp_day: false,
      hour: 0,
      cmp_hour: false,
      am_pm: false,
      min: 0,
      cmp_min: false
    }
  }

  pub fn read(&self, byte: u8) -> u8 {
    match byte {
      0 => (self.cmp_day as u8) << 7 | self.day,
      1 => (self.cmp_hour as u8) << 7 | (self.am_pm as u8) << 6 | self.hour,
      2 => (self.cmp_min as u8) << 7 | self.min,
      _ => unreachable!()
    }
  }

  pub fn write(&mut self, value: u8, byte: u8) {
    match byte {
      0 => {
        self.day = value & 0x7;
        self.cmp_day = (value >> 7) & 0b1 == 1;
      }
      1 => {
        self.hour = value & 0x3f;
        self.cmp_hour = (value >> 7) & 0b1 == 1;
        self.am_pm = (value >> 6) & 0b1 == 1;
      }
      2 => {
        self.min = value & 0x7f;
        self.cmp_min = (value >> 7) & 0b1 == 1;
      }
      _ => unreachable!()
    }
  }
}