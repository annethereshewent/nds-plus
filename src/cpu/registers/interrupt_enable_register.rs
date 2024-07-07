/*
  0     LCD V-Blank
  1     LCD H-Blank
  2     LCD V-Counter Match
  3     Timer 0 Overflow
  4     Timer 1 Overflow
  5     Timer 2 Overflow
  6     Timer 3 Overflow
  7     NDS7 only: SIO/RCNT/RTC (Real Time Clock)
  8     DMA 0
  9     DMA 1
  10    DMA 2
  11    DMA 3
  12    Keypad
  13    GBA-Slot (external IRQ source) / DSi: None such
  14    Not used                       / DSi9: NDS-Slot Card change?
  15    Not used                       / DSi: dito for 2nd NDS-Slot?
  16    IPC Sync
  17    IPC Send FIFO Empty
  18    IPC Recv FIFO Not Empty
  19    NDS-Slot Game Card Data Transfer Completion
  20    NDS-Slot Game Card IREQ_MC
  21    NDS9 only: Geometry Command FIFO
  22    NDS7 only: Screens unfolding
  23    NDS7 only: SPI bus
  24    NDS7 only: Wifi    / DSi9: XpertTeak DSP */

bitflags! {
  #[derive(Copy, Clone, Debug)]
  pub struct InterruptEnableRegister: u32 {
    const VBLANK = 0b1;
    const HBLANK = 0b1 << 1;
    const VCOUNTER_MATCH = 0b1 << 2;
    const TIMER_0_OVERFLOW = 0b1 << 3;
    const TIMER_1_OVERFLOW = 0b1 << 4;
    const TIMER_2_OVERFLOW = 0b1 << 5;
    const TIMER_3_OVERFLOW = 0b1 << 6;
    const SIO_RTC = 0b1 << 7;
    const DMA0 = 0b1 << 8;
    const DMA1 = 0b1 << 9;
    const DMA2 = 0b1 << 10;
    const DMA3 = 0b1 << 11;
    const KEYPAD = 0b1 << 12;
    const GAMEPACK = 0b1 << 13;
    const IPC_SEND = 0b1 << 16;
    const IPC_SEND_FIFO_EMPTY = 0b1 << 17;
    const IPC_RECV_FIFO_NOT_EMPTY = 0b1 << 18;
    const GAME_CARD_TRANSFER_COMPLETE = 0b1 << 19;
    const GAME_CARD_IREQ_MC = 0b1 << 20;
    const GEOMETRY_COMMAND = 0b1 << 21;

  }
}

pub const FLAG_VBLANK: u32 = 0b1;
pub const FLAG_HBLANK: u32 = 0b1 << 1;
pub const FLAG_VCOUNTER_MATCH: u32 = 0b1 << 2;
pub const FLAG_TIMER_0_OVERFLOW: u32 = 0b1 << 3;
pub const FLAG_TIMER_1_OVERFLOW: u32 = 0b1 << 4;
pub const FLAG_TIMER_2_OVERFLOW: u32 = 0b1 << 5;
pub const FLAG_TIMER_3_OVERFLOW: u32 = 0b1 << 6;
pub const FLAG_SIO_RTC: u32 = 0b1 << 7;
pub const FLAG_DMA0: u32 = 0b1 << 8;
pub const FLAG_DMA1: u32 = 0b1 << 9;
pub const FLAG_DMA2: u32 = 0b1 << 10;
pub const FLAG_DMA3: u32 = 0b1 << 11;
pub const FLAG_KEYPAD: u32 = 0b1 << 12;
pub const FLAG_GAMEPACK: u32 = 0b1 << 13;
pub const FLAG_IPC_SEND: u32 = 0b1 << 16;
pub const FLAG_IPC_SEND_FIFO_EMPTY: u32 = 0b1 << 17;
pub const FLAG_IPC_RECV_FIFO_NOT_EMPTY: u32 = 0b1 << 18;
pub const FLAG_GAME_CARD_TRANSFER_COMPLETE: u32 = 0b1 << 19;
pub const FLAG_GAME_CARD_IREQ_MC: u32 = 0b1 << 20;
pub const FLAG_GEOMETRY_COMMAND: u32 = 0b1 << 21;