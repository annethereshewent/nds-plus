use super::CPU;

pub enum ThumbInstructionType {
  AddSubtract,
  MoveShiftedRegister,
  MoveCompareAddSub,
  ALUOps,
  HiRegisterOps,
  PCRelativeLoad,
  LoadStoreRegisterOffset,
  LoadStoreSignedByteHalf,
  LoadStoreImmOffset,
  LoadStoreHalf,
  SPRelativeLoadStore,
  LoadAddress,
  AddOffsetToSp,
  PushPopRegisters,
  MultipleStoreLoad,
  SoftwareInterrupt,
  ConditionalBranch,
  UnconditionalBranch,
  LongBranchLinkExchange,
  LongBranchLink,
  Undefined
}

impl<const IS_ARM9: bool> CPU<IS_ARM9> {
  fn decode_thumb_mnemonic(&mut self, format: u16) -> ThumbInstructionType {
    use ThumbInstructionType::*;

    if format & 0b11111000 == 0b00011000 {
      AddSubtract
    } else if format & 0b11100000 == 0 {
      MoveShiftedRegister
    } else if format & 0b11100000 == 0b00100000 {
      MoveCompareAddSub
    } else if format & 0b11111100 == 0b01000000 {
      ALUOps
    } else if format & 0b11111100 == 0b01000100 {
      HiRegisterOps
    } else if format & 0b11111000 == 0b01001000 {
      PCRelativeLoad
    } else if format & 0b11110010 == 0b01010000 {
      LoadStoreRegisterOffset
    } else if format & 0b11110010 == 0b01010010 {
      LoadStoreSignedByteHalf
    } else if format & 0b11100000 == 0b01100000 {
      LoadStoreImmOffset
    } else if format & 0b11110000 == 0b10000000 {
      LoadStoreHalf
    } else if format & 0b11110000 == 0b10010000 {
      SPRelativeLoadStore
    } else if format & 0b11110000 == 0b10100000 {
      LoadAddress
    } else if format == 0b10110000 {
      AddOffsetToSp
    } else if format & 0b11110110 == 0b10110100 {
      PushPopRegisters
    } else if format & 0b11110000 == 0b11000000 {
      MultipleStoreLoad
    } else if format == 0b11011111 {
      SoftwareInterrupt
    } else if format & 0b11110000 == 0b11010000 {
      ConditionalBranch
    } else if format & 0b11111000 == 0b11101000 {
      LongBranchLinkExchange
    } else if format & 0b11111000 == 0b11100000 {
      UnconditionalBranch
    } else if format & 0b11110000 == 0b11110000 {
      LongBranchLink
    } else {
      Undefined
    }
  }

  pub fn populate_thumb_disassembly_lut(&mut self) {
    for i in 0..256 {
      let instr_fn = self.decode_thumb_mnemonic(i);
      self.disassembly_thumb_lut.push(instr_fn);
    }
  }
  pub fn disassemble_thumb_instr(&self, instr: u16) -> String {
    use ThumbInstructionType::*;

    match self.disassembly_thumb_lut[(instr >> 8) as usize] {
      AddSubtract => self.decode_add_subtract(instr),
      MoveShiftedRegister => self.decode_move_shifted_register(instr),
      MoveCompareAddSub => self.decode_move_compare_add_sub(instr),
      ALUOps => self.decode_alu_ops(instr),
      HiRegisterOps => self.decode_hi_register_ops(instr),
      PCRelativeLoad => self.decode_pc_relative_load(instr),
      LoadStoreRegisterOffset => self.decode_load_store_register_offset(instr),
      LoadStoreSignedByteHalf => self.decode_load_store_signed_byte_half(instr),
      LoadStoreImmOffset => self.decode_load_store_imm_offfset(instr),
      LoadStoreHalf => self.decode_load_store_half(instr),
      SPRelativeLoadStore => self.decode_sp_relative_load_store(instr),
      LoadAddress => self.decode_load_address(instr),
      AddOffsetToSp => self.decode_add_offset_to_sp(instr) ,
      PushPopRegisters => self.decode_push_pop_registers(instr),
      MultipleStoreLoad => self.decode_multiple_store_load(instr),
      SoftwareInterrupt => "SWI".to_string(),
      ConditionalBranch => self.decode_conditional_branch(instr),
      UnconditionalBranch => self.decode_unconditional_branch(instr),
      LongBranchLinkExchange => self.decode_long_branch_link_exchange(instr),
      LongBranchLink => self.decode_long_branch_link(instr),
      Undefined => "".to_string()
    }
  }

  fn decode_add_subtract(&self, instr: u16) -> String {
    let mut decoded = "".to_string();

    let op_code = (instr >> 9) & 0b1;
    let rn_offset = (instr >> 6) & 0x7;
    let is_immediate = (instr >> 10) & 0b1 == 1;

    let rs = (instr >> 3) & 0x7;
    let rd = instr & 0x7;

    if op_code == 0 {
      decoded += "ADD";
    } else {
      decoded += "SUB";
    }

    decoded += &format!(" r{rd}, r{rs}");

    if is_immediate {
      decoded += &format!(", {rn_offset}")
    } else {
      decoded += &format!(", r{rn_offset}")
    }

    decoded
  }

  fn decode_move_shifted_register(&self, instr: u16) -> String {
    let op_code = ((instr >> 11) & 0x3) as u8;
    let offset5 = ((instr >> 6) & 0x1f) as u8;
    let rs = ((instr >> 3) & 0x7) as u8;
    let rd = (instr & 0x7) as u8;

   let mut decoded = match op_code {
      0 => "LSL",
      1 => "LSR",
      2 => "ASR",
      _ => panic!("invalid option given")
    }.to_string();

    decoded += &format!(" r{rd}, r{rs}, #{:x}", offset5);

    decoded
  }

  fn decode_move_compare_add_sub(&self, instr: u16) -> String {
    let op_code = (instr >> 11) & 0x3;
    let rd = (instr >> 8) & 0x7;
    let offset = instr & 0xff;

    let mut decoded = self.get_move_compare_op_name(op_code).to_string();

    decoded += &format!(" r{rd}, #{:x}", offset);

    decoded
  }

  fn decode_alu_ops(&self, instr: u16) -> String {
    let op_code = (instr >> 6) & 0xf;
    let rs = (instr >> 3) & 0x7;
    let rd = instr & 0x7;

    let mut decoded = self.get_alu_op_name(op_code).to_string();

    decoded += &format!(" r{rd}, r{rs}");

    decoded
  }

  fn decode_hi_register_ops(&self, instr: u16) -> String {
    let op_code = (instr >> 8) & 0x3;
    let h1 = (instr >> 7) & 0b1;
    let h2 = (instr >> 6) & 0b1;

    let mut rs = (instr >> 3) & 0x7;
    let mut rd = instr & 0x7;

    if h1 == 1 {
      rd += 8;
    }
    if h2 == 1 {
      rs += 8;
    }

    let mut decoded = self.get_hi_register_op_name(op_code).to_string();

    if decoded == "BX" {
      if h1 == 1 && IS_ARM9 {
        decoded = format!("BLX r{rs}");
      } else {
        decoded += &format!(" r{rs}");
      }
    } else {
      decoded += &format!(" r{rd}, r{rs}");
    }

    decoded
  }

  fn decode_pc_relative_load(&self, instr: u16) -> String {
    let rd = (instr >> 8) & 0x7;
    let immediate = (instr & 0xff) << 2;

    let decoded = format!("LDR r{rd} [pc, #{:x}]", immediate);

    decoded
  }

  fn decode_load_store_register_offset(&self, instr: u16) -> String {
    let mut decoded = "".to_string();

    let rb = (instr >> 3) & 0x7;
    let ro = (instr >> 6) & 0x7;

    let b = (instr >> 10) & 0b1;

    // Rd,[Rb,Ro]
    let l = (instr >> 11) & 0b1;
    let rd = instr & 0x7;

    if l == 0 {
      decoded += "STR";
    } else {
      decoded += "LDR";
    }

    if b == 1 {
      decoded += "B";
    }

    decoded += &format!(" r{rd}, [r{rb},r{ro}]");

    decoded
  }

  fn decode_load_store_signed_byte_half(&self, instr: u16) -> String {
    let op_code = instr >> 10 & 0x3;

    let ro = (instr >> 6) & 0x7;
    let rb = (instr >> 3) & 0x7;
    let rd = instr & 0x7;

    let mut decoded = match op_code {
      0 => "STRH",
      1 => "LDSB",
      2 => "LDRH",
      3 => "LDSH",
      _ => unreachable!()
    }.to_string();

    decoded += &format!(" r{rd}, [r{rb},r{ro}]");

    decoded
  }

  fn decode_load_store_imm_offfset(&self, instr: u16) -> String {
    let mut decoded = "".to_string();

    let l = (instr >> 11) & 0b1;
    let b = (instr >> 12) & 0b1;

    if l == 0 {
      decoded += "STR";
    } else {
      decoded += "LDR";
    }

    if b == 1 {
      decoded += "B";
    }

    let offset = if b == 1 {
      (instr >> 6) & 0x1f
    } else {
      ((instr >> 6) & 0x1f) << 2
    };

    let rb = (instr >> 3) & 0x7;
    let rd = instr & 0x7;

    decoded += &format!(" r{rd}, [r{rb}, #{:x}]", offset);

    decoded
  }

  fn decode_load_store_half(&self, instr: u16) -> String {
    let mut decoded = "".to_string();

    let l = (instr >> 11) & 0b1;
    let offset = (((instr >> 6) & 0x1f) << 1) as i32;
    let rb = (instr >> 3) & 0x7;
    let rd = instr & 0x7;

    if l == 0 {
      decoded += "STRH";
    } else {
      decoded += "LDRH";
    }

    decoded += &format!(" r{rd}, [r{rb}, #{:x}]", offset);

    decoded
  }

  fn decode_sp_relative_load_store(&self, instr: u16) -> String {
    let mut decoded = "".to_string();

    let l = (instr >> 11) & 0b1;
    let rd = (instr >> 8) & 0x7;
    let word8 = (instr & 0xff) << 2;

    if l == 0 {
      decoded += "STR";
    } else {
      decoded += "LDR";
    }

    decoded += &format!(" r{rd}, [sp, #{:x}]", word8);

    decoded
  }

  fn decode_load_address(&self, instr: u16) -> String {
    let mut decoded = "ADD".to_string();

    let sp = (instr >> 11) & 0b1;
    let rd = (instr >> 8) & 0x7;
    let word8 = (instr & 0xff) << 2;

    decoded += &format!(" r{rd}");

    if sp == 0 {
      decoded += ", pc";
    } else {
      decoded += ", sp";
    }

    decoded += &format!(", #{:x}", word8);

    decoded
  }

  fn decode_add_offset_to_sp(&self, instr: u16) -> String {
    let mut decoded = "ADD sp".to_string();

    let s = (instr >> 7) & 0b1;
    let mut sword7 = ((instr & 0x7f) << 2) as i32;

    if s == 1 {
      sword7 *= -1;
    }

    decoded += &format!(", #{:x}", sword7);

    decoded
  }

  fn decode_push_pop_registers(&self, instr: u16) -> String {
    let mut decoded = "".to_string();

    let l = (instr >> 11) & 0b1;
    let r = (instr >> 8) & 0b1;
    let register_list = instr & 0xff;

    let mut registers = Vec::new();

    for i in 0..8 {
      if (register_list >> i) & 0b1 == 1 {
        registers.push(format!("r{i}"));
      }
    }

    if l == 0 {
      decoded += "PUSH";

      if r == 1 {
        registers.push("lr".to_string());
      }
    } else {
      decoded += "POP";
      if r == 1 {
        registers.push("pc".to_string());
      }
    }

    decoded += " {";

    decoded += &registers.join(", ");

    decoded += "}";

    decoded
  }

  fn decode_multiple_store_load(&self, instr: u16) -> String {
    let mut decoded = "".to_string();

    let l = (instr >> 11) & 0b1;
    let rb = (instr >> 8) & 0x7;
    let rlist = instr & 0xff;

    if l == 0 {
      decoded += "STMIA";
    } else {
      decoded += "LDMIA";
    }

    let mut registers = Vec::new();

    for i in 0..8 {
      if (rlist >> i) & 0b1 == 1 {
        registers.push(format!("r{i}"));
      }
    }

    decoded += &format!(" r{rb}!");

    decoded += " {";

    decoded += &registers.join(", ");

    decoded += "}";

    decoded
  }

  fn decode_conditional_branch(&self, instr: u16) -> String {
    let mut decoded = "B".to_string();

    let cond = (instr >> 8) & 0xf;

    decoded += &Self::parse_condition(cond as u32);

    let signed_offset = ((((instr & 0xff) as u32) << 24) as i32) >> 23;

    decoded += &format!(" {:x}", signed_offset);

    decoded
  }

  fn decode_unconditional_branch(&self, instr: u16) -> String {
    let mut decoded = "B".to_string();

    let address = ((((instr & 0x7ff) as i32) << 21)) >> 20;

    decoded += &format!(" {:x}", address);

    decoded
  }

  fn decode_long_branch_link_exchange(&self, instr: u16) -> String {
    let mut decoded = "BLX".to_string();

    let offset = (instr & 0x7ff) as i32;

    decoded += &format!(" {:x}", offset);

    decoded
  }

  fn decode_long_branch_link(&self, instr: u16) -> String {
    let h = (instr >> 11) & 0b1;
    let offset = (instr & 0x7ff) as i32;

    let decoded = format!("BL({h}) {:x}", offset);

    decoded
  }

  fn get_move_compare_op_name(&self, op_code: u16) -> &str {
    match op_code {
      0 => "MOV",
      1 => "CMP",
      2 => "ADD",
      3 => "SUB",
      _ => unreachable!("impossible")
    }
  }

  fn get_hi_register_op_name(&self, op_code: u16) -> &str {
    match op_code {
      0 => "ADD",
      1 => "CMP",
      2 => "MOV",
      3 => "BX",
      _ => unreachable!()
    }
  }

  fn get_alu_op_name(&self, op_code: u16) -> &str {
    match op_code {
      0 => "AND",
      1 => "EOR",
      2 => "LSL",
      3 => "LSR",
      4 => "ASR",
      5 => "ADC",
      6 => "SBC",
      7 => "ROR",
      8 => "TST",
      9 => "NEG",
      10 => "CMP",
      11 => "CMN",
      12 => "ORR",
      13 => "MUL",
      14 => "BIC",
      15 => "MVN",
      _ => unreachable!()
    }
  }
}