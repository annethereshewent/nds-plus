use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Default)]
pub enum Baudrate {
  #[default]
  Mhz4 = 0,
  Mhz2 = 1,
  Mhz1 = 2,
  Mhz512 = 3
}

#[derive(Copy, Clone, PartialEq, Default)]
pub enum SlotMode {
  #[default]
  ParallelRom = 0,
  SerialSPI = 1
}

#[derive(Default)]
pub struct SPICNT {
  pub baudrate: Baudrate,
  pub hold_chipselect: bool,
  pub spi_busy: bool,
  pub nds_slot_mode: SlotMode,
  pub transfer_ready_irq: bool,
  pub nds_slot_enable: bool
}

impl SPICNT {
  pub fn new() -> Self {
    Self {
      baudrate: Baudrate::Mhz4,
      hold_chipselect: false,
      spi_busy: false,
      nds_slot_mode: SlotMode::ParallelRom,
      transfer_ready_irq: false,
      nds_slot_enable: false
    }
  }

  pub fn write(&mut self, value: u16, has_access: bool, mask: Option<u16>) {

    if has_access {
      let mut val = 0;

      if let Some(mask) = mask {
        val = self.read(has_access) & mask;
      }

      val |= value;

      self.baudrate = match val & 0x3 {
        0 => Baudrate::Mhz4,
        1 => Baudrate::Mhz2,
        2 => Baudrate::Mhz1,
        3 => Baudrate::Mhz512,
        _ => unreachable!()
      };

      self.hold_chipselect = (val >> 6) & 0b1 == 1;
      self.nds_slot_mode = match (val >> 13) & 0b1 {
        0 => SlotMode::ParallelRom,
        1 => SlotMode::SerialSPI,
        _ => unreachable!()
      };

      self.transfer_ready_irq = (val >> 14) & 0b1 == 1;
      self.nds_slot_enable = (val >> 15) & 0b1 == 1;
    }
  }
  /*
    0-1   SPI Baudrate        (0=4MHz/Default, 1=2MHz, 2=1MHz, 3=512KHz)
    2-5   Not used            (always zero)
    6     SPI Hold Chipselect (0=Deselect after transfer, 1=Keep selected)
    7     SPI Busy            (0=Ready, 1=Busy) (presumably Read-only)
    8-12  Not used            (always zero)
    13    NDS Slot Mode       (0=Parallel/ROM, 1=Serial/SPI-Backup)
    14    Transfer Ready IRQ  (0=Disable, 1=Enable) (for ROM, not for AUXSPI)
    15    NDS Slot Enable     (0=Disable, 1=Enable) (for both ROM and AUXSPI)
  */
  pub fn read(&self, has_access: bool) -> u16 {
    if has_access {
      return self.baudrate as u16 |
        (self.hold_chipselect as u16) << 6 |
        (self.spi_busy as u16) << 7 |
        (self.nds_slot_mode as u16) << 13 |
        (self.transfer_ready_irq as u16) << 14 |
        (self.nds_slot_enable as u16) << 15
    }

    0
  }
}