bitflags! {
  #[derive(Clone, Copy)]
  pub struct KeyInputRegister: u16 {
    const ButtonA = 0b1;
    const ButtonB = 0b1 << 1;
    const Select = 0b1 << 2;
    const Start = 0b1 << 3;
    const Right = 0b1 << 4;
    const Left = 0b1 << 5;
    const Up = 0b1 << 6;
    const Down = 0b1 << 7;
    const ButtonR = 0b1 << 8;
    const ButtonL = 0b1 << 9;
  }
}

