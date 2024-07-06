#[derive(Clone, Copy)]
pub enum DeviceSelect {
  PowerManager = 0,
  Firmware = 1,
  Touchscreen = 2
}

#[derive(Copy, Clone)]
pub enum TransferSize {
  Bit8 = 0,
  Bit16 = 1
}

pub struct SPIControlRegister {
  pub baudrate: u16,
  pub busy: bool,
  pub device: DeviceSelect,
  pub transfer_size: TransferSize,
  pub chipselect_hold: bool,
  pub interrupt_request: bool,
  pub spi_bus_enabled: bool
}

impl SPIControlRegister {
  pub fn new() -> Self {
    Self {
      baudrate: 0,
      busy: false,
      device: DeviceSelect::PowerManager,
      transfer_size: TransferSize::Bit8,
      chipselect_hold: false,
      interrupt_request: false,
      spi_bus_enabled: false
    }
  }

  pub fn write(&mut self, val: u16) {
    self.baudrate = val & 0x3;
    self.device = match (val >> 8) & 0x3 {
      0 => DeviceSelect::PowerManager,
      1 => DeviceSelect::Firmware,
      2 => DeviceSelect::Touchscreen,
      _ => panic!("invalid option given for device select: {}", (val >> 8) & 0x3)
    };

    self.transfer_size = match (val >> 10) & 0b1 {
      0 => TransferSize::Bit8,
      1 => TransferSize::Bit16,
      _ => unreachable!()
    };

    self.chipselect_hold = (val >> 11) & 0b1 == 1;
    self.interrupt_request = (val >> 14) & 0b1 == 1;
    self.spi_bus_enabled = (val >> 15) & 0b1 == 1;
  }

  pub fn read(&self) -> u16 {
    self.baudrate as u16 |
      (self.busy as u16) << 7 |
      (self.device as u16) << 8 |
      (self.transfer_size as u16) << 10 |
      (self.chipselect_hold as u16) << 11 |
      (self.interrupt_request as u16) << 14 |
      (self.spi_bus_enabled as u16) << 15
  }
}