use super::CPU;

impl<const IS_ARM9: bool> CPU<IS_ARM9> {
  pub fn lsl(&mut self, operand: u32, shift: u32, carry: &mut bool) -> u32 {
    if shift < 32 {
      if shift != 0 {
        let carry_shift = 32 - shift;
        *carry = (operand >> carry_shift) & 0b1 == 1;
      }

      if shift < 32 { operand << shift } else { 0 }
    } else if shift == 32 {
      *carry = operand & 0b1 == 1;
      0
    } else {
      *carry = false;
      0
    }
  }

  pub fn ror(&mut self, immediate: u32, amount: u8, is_immediate: bool, rrx: bool, carry: &mut bool) -> u32 {
    if amount != 0 {
      let amount = amount % 32;

      let result = immediate.rotate_right(amount as u32);

      *carry = (result >> 31) & 0b1 == 1;

      result
    } else if is_immediate && rrx {
      self.rrx(immediate, carry)
    } else {
      immediate
    }
  }

  fn rrx(&mut self, operand: u32, carry: &mut bool) -> u32 {
    let to_carry = if *carry { 1 } else { 0 };

    *carry = operand & 0b1 != 0;

    ((operand >> 1) as i32 | (to_carry << 31)) as u32
  }

  pub fn lsr(&mut self, operand: u32, shift: u32, immediate: bool, carry: &mut bool) -> u32 {
    if shift != 0 {
      if shift < 32 {
       *carry = ((operand >> (shift - 1)) & 0b1) == 1;
        operand >> shift
      } else if shift == 32 {
        *carry = operand >> 31 == 1;
        0
      } else {
        *carry = false;
        0
      }
    } else if immediate {
      *carry = operand >> 31 == 1;
      0
    } else {
      operand
    }
  }

  pub fn asr(&mut self, operand: u32, shift: u32, immediate: bool, carry: &mut bool) -> u32 {
    let shift = if immediate && shift == 0 { 32 } else { shift };

    match shift  {
      0 => operand,
      x if x < 32 => {
        *carry = operand.wrapping_shr(shift as u32 - 1) & 0b1 == 1;
        (operand as i32).wrapping_shr(shift as u32) as u32
      }
      _ => {
        if operand >> 31 == 1 {
          *carry = true;
          0xffff_ffff
        } else {
          *carry = false;
          0
        }
      }
    }
  }
}