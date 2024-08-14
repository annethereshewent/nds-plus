use std::ops::BitOrAssign;

use num::{FromPrimitive, NumCast, PrimInt, Unsigned};

pub trait Number:
  Unsigned + PrimInt + NumCast + FromPrimitive + std::fmt::LowerHex + BitOrAssign
{

}

impl Number for u8 {}
impl Number for u16 {}
impl Number for u32 {}
impl Number for u64 {}