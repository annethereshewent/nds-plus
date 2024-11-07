use super::{PSRRegister, CPU, PC_REGISTER};

enum ArmInstructionType {
  Multiply,
  MultiplyLong,
  SignedHalfwordMultiply,
  SingleDataSwap,
  BranchAndExchange,
  HalfwordDataTransferRegister,
  HalfwordDataTransferImmediate,
  QALU,
  CoprocessorTransfer,
  CLZ,
  DataProcessing,
  MSR,
  MRS,
  SingleDataTransfer,
  BlockDataTransfer,
  Branch,
  SoftwareInterrupt,
  Undefined
}

impl<const IS_ARM9: bool> CPU<IS_ARM9> {
  pub fn disassemble_arm_instr(&self, instr: u32) -> String {
    use ArmInstructionType::*;
    match self.decode_arm_mnemonic((instr >> 20 & 0xff) as u16, ((instr >> 4) & 0xf) as u16) {
      Multiply => self.decode_multiply(instr),
      MultiplyLong => self.decode_multiply_long(instr),
      SignedHalfwordMultiply => self.decode_signed_multiply(instr),
      SingleDataSwap => self.decode_swap(instr),
      BranchAndExchange => self.decode_branch_and_exchange(instr),
      HalfwordDataTransferRegister => self.decode_halfword_data_transfer(instr, true),
      HalfwordDataTransferImmediate => self.decode_halfword_data_transfer(instr, false),
      QALU => self.decode_qalu(instr),
      CoprocessorTransfer => self.decode_cop_transfer(instr),
      CLZ => self.decode_clz(instr),
      DataProcessing => self.decode_data_processing(instr),
      MSR => self.decode_msr(instr),
      MRS => self.decode_mrs(instr),
      SingleDataTransfer => self.decode_single_transfer(instr),
      BlockDataTransfer => self.decode_block_transfer(instr),
      Branch => self.decode_branch(instr),
      SoftwareInterrupt => "SWI".to_string(),
      Undefined => "".to_string()
    }
  }

  fn decode_arm_mnemonic(&self, upper: u16, lower: u16) -> ArmInstructionType {
    use ArmInstructionType::*;
    if upper & 0b11111100 == 0 && lower == 0b1001 {
      Multiply
    } else if upper & 0b11111000 == 0b00001000 && lower == 0b1001 {
      MultiplyLong
    } else if upper & 0b11111001 == 0b00010000 && lower & 0b1001 == 0b1000 {
      SignedHalfwordMultiply
    } else if upper & 0b11110011 == 0b00010000 && lower == 0b1001 {
      SingleDataSwap
    } else if upper == 0b00010010 && lower & 0b1101 == 0b1 {
      BranchAndExchange
    } else if upper & 0b11100100 == 0 && lower & 0b1001 == 0b1001 {
      HalfwordDataTransferRegister
    } else if upper & 0b11100100 == 0b00000100 && lower & 0b1001 == 0b1001 {
      HalfwordDataTransferImmediate
    } else if upper & 0b11111001 == 0b00010000 && lower == 0b0101 {
      QALU
    } else if upper & 0b11110000 == 0b11100000 && lower & 0b1 == 0b1 {
      CoprocessorTransfer
    } else if upper == 0b00010110 && lower == 0b1 {
      CLZ
    } else if upper & 0b11000000 == 0 {
      // check for psr transfer instructions as they are a subset of data processing
      let s = upper & 0b1;
      let op_code = (upper >> 1) & 0xf;

      let is_updating_flags_only = (op_code & 0b1100) == 0b1000;

      if s == 0 && is_updating_flags_only {
        if op_code & 0b1 == 0 {
          MSR
        } else {
          MRS
        }
      } else {
        DataProcessing
      }
    } else if upper & 0b11100000 == 0b01100000 && lower & 0b1 == 1 {
      // undefined instruction
      Undefined
    } else if upper & 0b11000000 == 0b01000000 {
      SingleDataTransfer
    } else if upper & 0b11100000 == 0b10000000 {
      BlockDataTransfer
    } else if upper & 0b11100000 == 0b10100000 {
      Branch
    } else if upper & 0b11110000 == 0b11110000 {
      SoftwareInterrupt
    }  else {
      Undefined
    }
  }

  fn decode_multiply(&self, instr: u32) -> String {
    let a = (instr >> 21) & 0b1;
    let rd = (instr >> 16) & 0xf;
    let rn = (instr >> 12) & 0xf;
    let rs = (instr >> 8) & 0xf;
    let rm = instr & 0xf;

    let mut decoded = "".to_string();

    if a == 1 {
      decoded += "MUL";
    } else {
      decoded += "MLA";
    }

    decoded = decoded + &format!(" r{rd}, r{rm}, r{rs}");

    if a == 1 {
      decoded = decoded + &format!(", {rn}");
    }

    decoded
  }

  fn decode_multiply_long(&self, instr: u32) -> String {
    let u = (instr >> 22) & 0b1;
    let a = (instr >> 21) & 0b1;
    let s = (instr >> 20) & 0b1;

    let rd_hi = (instr >> 16) & 0xf;
    let rd_low = (instr >> 12) & 0xf;
    let rs = (instr >> 8) & 0xf;
    let rm = instr & 0xf;

    let mut decoded = "".to_string();

    if u == 0 {
      decoded += "U";
    } else {
      decoded += "S";
    }

    if a == 0 {
      decoded += "MULL";
    } else {
      decoded += "MLAL";
    }

    decoded += &format!(" r{rd_low}, r{rd_hi}, r{rm}, r{rs}");

    decoded
  }

  fn decode_signed_multiply(&self, instr: u32) -> String {
    let x = (instr >> 5) & 0b1;

    let rd = (instr >> 16) & 0xf;
    let rn = (instr >> 12) & 0xf;
    let rs = (instr >> 8) & 0xf;
    let rm = instr & 0xf;

    let opcode = (instr >> 21) & 0b11;

    let mut decoded = format!("r{rd}, r{rm}, r{rs}, r{rn}");

    /*
        1000b: SMLAxy{cond}   Rd,Rm,Rs,Rn     ;Rd=HalfRm*HalfRs+Rn
        1001b: SMLAWy{cond}   Rd,Rm,Rs,Rn     ;Rd=(Rm*HalfRs)/10000h+Rn
        1001b: SMULWy{cond}   Rd,Rm,Rs        ;Rd=(Rm*HalfRs)/10000h
        1010b: SMLALxy{cond}  RdLo,RdHi,Rm,Rs ;RdHiLo=RdHiLo+HalfRm*HalfRs
        1011b: SMULxy{cond}   Rd,Rm,Rs        ;Rd=HalfRm*HalfRs
    */

    match opcode {
      0b00 => {
        decoded = format!("SMLAxy {decoded}");
      }
      0b01 => {
        // so this op code has two different commands that are dependent on x. if x == 1, SMULW, if x == 0, SMLAW
        if x == 0 {
          // Rd=(Rm*HalfRs)/10000h+Rn SMLAW
          decoded = format!("SMLAWy {decoded}");
        } else {
          // Rd=(Rm*HalfRs)/10000h SMULW
          decoded = format!("SMULWy r{rd}, r{rm}, r{rs}");
        }
      }
      0b10 => {
        // SMLALxy
        // RdHiLo=RdHiLo+HalfRm*HalfRs
        let rd_hi = rd;
        let rd_lo = rn;

        decoded = format!("SMLALxy r{rd_lo}, r{rd_hi}, r{rm}, r{rs}");
      }
      0b11 => {
        // SMULxy
        // Rd=HalfRm*HalfRs
        decoded = format!("SMULxy r{rd}, r{rm}, r{rs}");
      }
      _ => panic!("unreachable")
    }

    decoded
  }

  fn decode_swap(&self, instr: u32) -> String {
    let mut decoded = "SWP".to_string();

    let b = (instr >> 22) & 0b1;
    let rn = (instr >> 16) & 0xf;
    let rd = (instr >> 12) & 0xf;
    let rm = instr & 0xf;

    if b == 1 {
      decoded += "B";
    }

    decoded += &format!(" r{rd}, r{rm}, [r{rn}]");

    decoded
  }

  fn decode_branch_and_exchange(&self, instr: u32) -> String {
    let mut decoded = "".to_string();

    let rn = instr & 0xf;
    let l = (instr >> 5) & 0b1;

    if l == 0 {
      decoded = format!("BX");
    } else {
      decoded = format!("BLX");
    }

    decoded += &Self::parse_condition(instr);


    decoded += &format!(" r{rn}");

    decoded
  }

  fn decode_halfword_data_transfer(&self, instr: u32, is_register: bool) -> String {
    let mut decoded = "".to_string();

    let rm = instr & 0xf;

    let offset = if is_register {
      self.get_register(rm as usize)
    } else {
      let offset_high = (instr >> 8) & 0xf;
      let offset_low = instr & 0xf;

      offset_high << 4 | offset_low
    };

    let sh = (instr >> 5) & 0b11;
    let rd = (instr >> 12) & 0xf;
    let rn = (instr >> 16) & 0xf;

    let l = (instr >> 20) & 0b1;
    let w = (instr >> 21) & 0b1;
    let u = (instr >> 23) & 0b1;
    let p = (instr >> 24) & 0b1;

    if l == 0 {
      match sh {
        1 => decoded = "STRH".to_string(),
        2 => decoded = "STRD".to_string(),
        3 => decoded = "LDRD".to_string(),
        _ => panic!("shouldn't happen")
      }
    } else {
      decoded = "LDR".to_string();

      match sh {
        1 => decoded += "H",
        2 => decoded += "SB",
        3 => decoded += "SH",
        _ => panic!("shouldn't happen")
      }
    }

    decoded += &format!(" r{rd}, [r{rn}");

    if offset != 0 {
      if p == 1 {
        let mut sign = "";
        if u == 0 {
          sign = "-";
        }

        let offset_str = if is_register { &format!(", {sign}r{rm}]") } else { &format!(", {sign}{:x}]", offset) };

        decoded += offset_str;
      } else {
        let mut sign = "";

        if u == 0 {
          sign = "-";
        }
        decoded += &format!("], {sign}{:x}", offset);
      }
    } else {
      decoded += "]";
    }

    if w == 1 {
      decoded += "!";
    }

    decoded
  }

  fn decode_qalu(&self, instr: u32) -> String {
    let mut decoded = "".to_string();

    decoded
  }

  fn decode_cop_transfer(&self, instr: u32) -> String {
    let mut decoded = "".to_string();

    decoded
  }

  fn decode_clz(&self, instr: u32) -> String {
    let mut decoded = "".to_string();

    decoded
  }

  fn decode_data_processing(&self, instr: u32) -> String {
    let mut decoded = "".to_string();

    let i = (instr >> 25) & 0b1;
    let op_code = (instr >> 21) & 0xf;
    let rn = (instr >> 16) & 0xf;
    let rd = (instr >> 12) & 0xf;

    let (op_name, num_args) = self.get_op_name(op_code as u8);

    let mut operand1 = self.get_register(rn as usize);

    let operand2 = if i == 1 {
      let immediate = instr & 0xff;
      let amount = (2 * ((instr >> 8) & 0xf)) as u8;

      // fuck you rust.
      self.ror_disassembly(immediate, amount, false, true)
    } else {
      // fuck you rust.
      self.get_data_processing_register_operand_diss(instr, rn, &mut operand1)
    };

    decoded += op_name;

    let args = if num_args == 3 {
      format!(" r{rd}, r{rn}, {:x}", operand2)
    } else {
      if op_name != "MOV" || op_name != "MVN" {
        format!(" r{rn}, {:x}", operand2)
      } else {
        format!(" r{rd}, {:x}", operand2)
      }
    };

    decoded += &args;

    decoded
  }

  fn decode_mrs(&self, instr: u32) -> String {
    let mut decoded = "".to_string();

    decoded
  }

  fn decode_msr(&self, instr: u32) -> String {
    let mut decoded = "".to_string();

    decoded
  }

  fn decode_single_transfer(&self, instr: u32) -> String {
    let mut decoded = "".to_string();

    decoded
  }

  fn decode_block_transfer(&self, instr: u32) -> String {
    let mut decoded = "".to_string();

    decoded
  }

  fn decode_branch(&self, instr: u32) -> String {
    let mut decoded = "".to_string();

    decoded
  }

  pub fn parse_condition(instr: u32) -> String {
    let condition = instr >> 28 & 0xf;
    let decoded = match condition {
      0 => "EQ",
      1 => "NE",
      2 => "CS",
      3 => "CC",
      4 => "MI",
      5 => "PL",
      6 => "VS",
      7 => "VC",
      8 => "HI",
      9 => "LS",
      10 => "GE",
      11 => "LT",
      12 => "GT",
      13 => "LE",
      14 => "",
      15 => "",
      _ => unreachable!("shouldn't happen")
    };

    decoded.to_string()
  }

  fn ror_disassembly(&self, immediate: u32, amount: u8, is_immediate: bool, rrx: bool) -> u32 {
    if amount != 0 {
      let amount = amount % 32;

      let result = immediate.rotate_right(amount as u32);

      result
    } else if is_immediate && rrx {
      self.rrx_disassembly(immediate)
    } else {
      immediate
    }
  }

  fn rrx_disassembly(&self, operand: u32) -> u32 {
    let carry = self.cpsr.contains(PSRRegister::CARRY);
    let to_carry = if carry { 1 } else { 0 };

    ((operand >> 1) as i32 | (to_carry << 31)) as u32
  }

  pub fn get_op_name(&self, op_code: u8) -> (&str, usize) {
    match op_code {
      0 => ("AND", 3),
      1 => ("EOR", 3),
      2 => ("SUB", 3),
      3 => ("RSB", 3),
      4 => ("ADD", 3),
      5 => ("ADC", 3),
      6 => ("SBC", 3),
      7 => ("RSC", 3),
      8 => ("TST", 2),
      9 => ("TEQ", 2),
      10 => ("CMP", 2),
      11 => ("CMN", 2),
      12 => ("ORR", 3),
      13 => ("MOV", 2),
      14 => ("BIC", 3),
      15 => ("MVN", 2),
      _ => unreachable!("can't happen")
    }
  }

  pub fn get_data_processing_register_operand_diss(&self, instr: u32, rn: u32, operand1: &mut u32) -> u32 {
    let shift_by_register = (instr >> 4) & 0b1 == 1;

    let mut immediate = true;

    let shift = if shift_by_register {
      immediate = false;

      if rn == PC_REGISTER as u32 {
        *operand1 += 4;
      }

      let rs = (instr >> 8) & 0xf;

      self.r[rs as usize] & 0xff
    } else {
      (instr >> 7) & 0x1f
    };

    let shift_type = (instr >> 5) & 0b11;

    let rm = instr & 0xf;

    let mut shifted_operand = self.get_register(rm as usize);

    if shift_by_register && rm == PC_REGISTER as u32 {
      shifted_operand += 4;
    }

    match shift_type {
      0 => self.lsl_disassembly(shifted_operand, shift),
      1 => self.lsr_disassembly(shifted_operand, shift, immediate),
      2 => self.asr_disassembly(shifted_operand, shift, immediate),
      3 => self.ror_disassembly(shifted_operand, shift as u8, immediate, true),
      _ => unreachable!("can't happen")
    }
  }

  pub fn lsl_disassembly(&self, operand: u32, shift: u32) -> u32 {
    if shift < 32 {
      operand << shift
    } else {
      0
    }
  }

  pub fn lsr_disassembly(&self, operand: u32, shift: u32, immediate: bool) -> u32 {
    if shift != 0 {
      if shift < 32 {
        operand >> shift
      } else {
        0
      }
    } else if immediate {
      0
    } else {
      operand
    }
  }

  pub fn asr_disassembly(&self, operand: u32, shift: u32, immediate: bool) -> u32 {
    let shift = if immediate && shift == 0 { 32 } else { shift };

    match shift  {
      0 => operand,
      x if x < 32 => {
        (operand as i32).wrapping_shr(shift as u32) as u32
      }
      _ => {
        if operand >> 31 == 1 {
          0xffff_ffff
        } else {
          0
        }
      }
    }
  }
}