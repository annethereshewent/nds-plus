#[derive(Copy, Clone, PartialEq)]
pub enum AccessRights {
  Arm9 = 0,
  Arm7 = 1
}

#[derive(Copy, Clone, PartialEq)]
pub enum AccessPriority {
  Arm9 = 0,
  Arm7 = 1
}

#[derive(Copy, Clone)]
pub enum MemoryMode {
  Asynchronous = 0,
  Synchronous = 1
}

pub struct ExternalMemory {
  pub arm7_exmem: ExternalMemoryControlRegister,
  pub arm9_exmem: ExternalMemoryControlRegister,
  pub gba_access_rights: AccessRights,
  pub nds_access_rights: AccessRights,
  pub memory_access_priority: AccessPriority,
  pub memory_mode: MemoryMode
}

impl ExternalMemory {
  pub fn new() -> Self {
    Self {
      arm7_exmem: ExternalMemoryControlRegister::new(),
      arm9_exmem: ExternalMemoryControlRegister::new(),
      gba_access_rights: AccessRights::Arm9,
      nds_access_rights: AccessRights::Arm9,
      memory_access_priority: AccessPriority::Arm9,
      memory_mode: MemoryMode::Synchronous
    }
  }

  pub fn read(&self, is_arm9: bool) -> u16 {
    let mut result = 0;

    result |= if is_arm9 {
      self.arm9_exmem.read()
    } else {
      self.arm7_exmem.read()
    };

    result |= (self.gba_access_rights as u16) << 7;
    result |= (self.nds_access_rights as u16) << 11;
    result |= (self.memory_mode as u16) << 14;
    result |= (self.memory_access_priority as u16) << 15;

    result
  }

  pub fn write(&mut self, is_arm9: bool, val: u16) {
    let cnt = if is_arm9 {
      &mut self.arm9_exmem
    } else {
      &mut self.arm7_exmem
    };

    cnt.write(val);

    if is_arm9 {
      // write to the upper bits also
      self.gba_access_rights = match (val >> 7) & 0b1 {
        0 => AccessRights::Arm9,
        1 => AccessRights::Arm7,
        _ => unreachable!()
      };

      self.nds_access_rights = match (val >> 11) & 0b1 {
        0 => AccessRights::Arm9,
        1 => AccessRights::Arm7,
        _ => unreachable!()
      };

      self.memory_mode = match (val >> 14) & 0b1 {
        0 => MemoryMode::Asynchronous,
        1 => MemoryMode::Synchronous,
        _ => unreachable!()
      };

      self.memory_access_priority = match (val >> 15) & 0b1 {
        0 => AccessPriority::Arm9,
        1 => AccessPriority::Arm7,
        _ => unreachable!()
      }
    }
  }
}

pub struct ExternalMemoryControlRegister {
  pub gba_sram_access_time: u16,
  pub gba_rom_1st_access: u16,
  pub gba_rom_2nd_access: u16,
  pub gba_phi: u16,
}

impl ExternalMemoryControlRegister {
  pub fn new() -> Self {
    Self {
      gba_sram_access_time: 0,
      gba_rom_1st_access: 0,
      gba_phi: 0,
      gba_rom_2nd_access: 0
    }
  }

  pub fn read(&self) -> u16 {
    let mut result = 0;

    result |= self.gba_sram_access_time;

    result |= self.gba_rom_1st_access << 2;
    result |= self.gba_rom_2nd_access << 4;
    result |= self.gba_phi << 5;


    result
  }

  pub fn write(&mut self, val: u16) {
    self.gba_sram_access_time = val & 0x3;
    self.gba_rom_1st_access = (val >> 2) & 0x3;
    self.gba_rom_2nd_access = (val >> 4) & 0b1;
    self.gba_phi = (val >> 5) &  0x3;
  }
}