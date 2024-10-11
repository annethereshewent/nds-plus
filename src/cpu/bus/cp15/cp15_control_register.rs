
/*
  0  MMU/PU Enable         (0=Disable, 1=Enable) (Fixed 0 if none)
  1  Alignment Fault Check (0=Disable, 1=Enable) (Fixed 0/1 if none/always on)
  2  Data/Unified Cache    (0=Disable, 1=Enable) (Fixed 0/1 if none/always on)
  3  Write Buffer          (0=Disable, 1=Enable) (Fixed 0/1 if none/always on)
  4  Exception Handling    (0=26bit, 1=32bit)    (Fixed 1 if always 32bit)
  5  26bit-address faults  (0=Enable, 1=Disable) (Fixed 1 if always 32bit)
  6  Abort Model (pre v4)  (0=Early, 1=Late Abort) (Fixed 1 if ARMv4 and up)
  7  Endian                (0=Little, 1=Big)     (Fixed 0/1 if fixed)
  8  System Protection bit (MMU-only)
  9  ROM Protection bit    (MMU-only)
  10 Implementation defined
  11 Branch Prediction     (0=Disable, 1=Enable)
  12 Instruction Cache     (0=Disable, 1=Enable) (ignored if Unified cache)
  13 Exception Vectors     (0=00000000h, 1=FFFF0000h)
  14 Cache Replacement     (0=Normal/PseudoRandom, 1=Predictable/RoundRobin)
  15 Pre-ARMv5 Mode        (0=Normal, 1=Pre ARMv5; LDM/LDR/POP_PC.Bit0/Thumb)
  16 DTCM Enable           (0=Disable, 1=Enable)
  17 DTCM Load Mode        (0=R/W, 1=DTCM Write-only)
  18 ITCM Enable           (0=Disable, 1=Enable)
  19 ITCM Load Mode        (0=R/W, 1=ITCM Write-only)
  20 Reserved              (0)
  21 Reserved              (0)
  22 Unaligned Access      (?=Enable unaligned access and mixed endian)
  23 Extended Page Table   (0=Subpage AP Bits Enabled, 1=Disabled)
  24 Reserved              (0)
  25 CPSR E on exceptions  (0=Clear E bit, 1=Set E bit)
  26 Reserved              (0)
  27 FIQ Behaviour         (0=Normal FIQ behaviour, 1=FIQs behave as NMFI)
  28 TEX Remap bit         (0=No remapping, 1=Remap registers used)
  29 Force AP              (0=Access Bit not used, 1=AP[0] used as Access bit)
  30 Reserved              (0)
  31 Reserved              (0)
  Various bits in this register may be read-only (fi
*/

bitflags! {
  #[derive(Default)]
  pub struct CP15ControlRegister: u32 {
    const PU_ENABLE = 1;
    const ALIGNMENT_FAULTCHECK = 1 << 1;
    const DATA_UNIFIED_CACHE = 1 << 2;
    const WRITE_BUFFER = 1 << 3;
    const EXCEPTION_HANDLING = 1 << 4;
    const ADDRESS_FAULTS_32 = 1 << 5;
    const ENDIAN_MODE = 1 << 7;
    const BRANCH_PREDICTION = 1 << 11;
    const INSTRUCTION_CACHE_ENABLE = 1 << 12;
    const EXCEPTION_VECTOR = 1 << 13;
    const CACHE_REPLACEMNET = 1 << 14;
    const PRE_ARMV5 = 1 << 15;
    const DTCM_ENABLE = 1 << 16;
    const DTCM_LOAD_MODE = 1 << 17;
    const ITCM_ENABLE = 1 << 18;
    const ITCM_LOAD_MODE = 1 << 19;
  }
}