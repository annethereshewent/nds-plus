bitflags! {
  #[derive(Copy, Clone)]
  pub struct InterruptEnableRegister: u16 {
    const VBLANK = 0b1;
    const HBLANK = 0b1 << 1;
    const VCOUNTER_MATCH = 0b1 << 2;
    const TIMER_0_OVERFLOW = 0b1 << 3;
    const TIMER_1_OVERFLOW = 0b1 << 4;
    const TIMER_2_OVERFLOW = 0b1 << 5;
    const TIMER_3_OVERFLOW = 0b1 << 6;
    const SERIAL_COMM = 0b1 << 7;
    const DMA0 = 0b1 << 8;
    const DMA1 = 0b1 << 9;
    const DMA2 = 0b1 << 10;
    const DMA3 = 0b1 << 11;
    const KEYPAD = 0b1 << 12;
    const GAMEPACK = 0b1 << 13;
  }
}

pub const FLAG_VBLANK: u16 = 0b1;
pub const FLAG_HBLANK: u16 = 0b1 << 1;
pub const FLAG_VCOUNTER_MATCH: u16 = 0b1 << 2;
pub const FLAG_TIMER_0_OVERFLOW: u16 = 0b1 << 3;
pub const FLAG_TIMER_1_OVERFLOW: u16 = 0b1 << 4;
pub const FLAG_TIMER_2_OVERFLOW: u16 = 0b1 << 5;
pub const FLAG_TIMER_3_OVERFLOW: u16 = 0b1 << 6;
pub const FLAG_SERIAL_COMM: u16 = 0b1 << 7;
pub const FLAG_DMA0: u16 = 0b1 << 8;
pub const FLAG_DMA1: u16 = 0b1 << 9;
pub const FLAG_DMA2: u16 = 0b1 << 10;
pub const FLAG_DMA3: u16 = 0b1 << 11;
pub const FLAG_KEYPAD: u16 = 0b1 << 12;
pub const FLAG_GAMEPACK: u16 = 0b1 << 13;