use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use super::interrupt_request_register::InterruptRequestRegister;

#[derive(Serialize, Deserialize)]
pub struct IPCFifoControlRegister {
  pub send_empty_irq: bool,
  pub fifo: VecDeque<u32>,

  pub receive_not_empty_irq: bool,

  pub error: bool,
  pub enabled: bool,

  pub previous_value: u32
}

pub const FIFO_CAPACITY: usize = 16;

impl IPCFifoControlRegister {
  pub fn new() -> Self {
    Self {
      send_empty_irq: false,
      receive_not_empty_irq: false,
      error: false,
      enabled: false,
      fifo: VecDeque::with_capacity(FIFO_CAPACITY),
      previous_value: 0
    }
  }
  /*
  4000184h - NDS9/NDS7 - IPCFIFOCNT - IPC Fifo Control Register (R/W)
  Bit   Dir  Expl.
  0     R    Send Fifo Empty Status      (0=Not Empty, 1=Empty)
  1     R    Send Fifo Full Status       (0=Not Full, 1=Full)
  2     R/W  Send Fifo Empty IRQ         (0=Disable, 1=Enable)
  3     W    Send Fifo Clear             (0=Nothing, 1=Flush Send Fifo)
  4-7   -    Not used
  8     R    Receive Fifo Empty          (0=Not Empty, 1=Empty)
  9     R    Receive Fifo Full           (0=Not Full, 1=Full)
  10    R/W  Receive Fifo Not Empty IRQ  (0=Disable, 1=Enable)
  11-13 -    Not used
  14    R/W  Error, Read Empty/Send Full (0=No Error, 1=Error/Acknowledge)
  15    R/W  Enable Send/Receive Fifo    (0=Disable, 1=Enable)
  */
  pub fn read(&self, receive_fifo: &mut VecDeque<u32>) -> u32 {
    (self.fifo.is_empty() as u32) |
      ((self.fifo.len() == FIFO_CAPACITY) as u32) << 1 |
      (self.send_empty_irq as u32) << 2 |
      (receive_fifo.is_empty() as u32) << 8 |
      ((receive_fifo.len() == FIFO_CAPACITY) as u32) << 9 |
      (self.receive_not_empty_irq as u32) << 10 |
      (self.error as u32) << 14 |
      (self.enabled as u32) << 15
  }

  pub fn write(
    &mut self,
    interrupt_request: &mut InterruptRequestRegister,
    receive_fifo: &mut VecDeque<u32>,
    val: u16
  ) {


    let previous_send_empty_irq = self.send_empty_irq;
    let previous_receive_not_empty_irq = self.receive_not_empty_irq;

    self.send_empty_irq = val >> 2 & 0b1 == 1;

    let send_fifo_clear = val >> 3 & 0b1 == 1;

    self.receive_not_empty_irq = val >> 10 & 0b1 == 1;
    self.error = self.error && val >> 14 & 0b1 == 0;

    self.enabled = val >> 15 & 0b1 == 1;

    if !receive_fifo.is_empty() && !previous_receive_not_empty_irq && self.receive_not_empty_irq {
      interrupt_request.insert(InterruptRequestRegister::IPC_RECV_FIFO_NOT_EMPTY);
    }

    if self.fifo.is_empty() && !previous_send_empty_irq  && self.send_empty_irq {
      interrupt_request.insert(InterruptRequestRegister::IPC_SEND_FIFO_EMPTY);
    }

    if send_fifo_clear {
      self.fifo.clear();
      self.previous_value = 0;
    }
  }
}