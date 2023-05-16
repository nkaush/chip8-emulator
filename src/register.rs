use std::fmt::{Display, Formatter};
use std::ops::{IndexMut, Index};

#[derive(Clone, Copy, Debug)]
pub enum VRegister { 
    V0 = 0x0,
    V1 = 0x1,
    V2 = 0x2,
    V3 = 0x3,
    V4 = 0x4,
    V5 = 0x5,
    V6 = 0x6,
    V7 = 0x7,
    V8 = 0x8,
    V9 = 0x9,
    VA = 0xa,
    VB = 0xb,
    VC = 0xc,
    VD = 0xd,
    VE = 0xe,
    VF = 0xf
}

#[derive(Debug)]
pub struct InvalidRegisterNumber(pub String);

impl TryFrom<u8> for VRegister {
    type Error = InvalidRegisterNumber;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use VRegister::*;

        match value {
            0x0 => Ok(V0),
            0x1 => Ok(V1),
            0x2 => Ok(V2),
            0x3 => Ok(V3),
            0x4 => Ok(V4),
            0x5 => Ok(V5),
            0x6 => Ok(V6),
            0x7 => Ok(V7),
            0x8 => Ok(V8),
            0x9 => Ok(V9),
            0xa => Ok(VA),
            0xb => Ok(VB),
            0xc => Ok(VC),
            0xd => Ok(VD),
            0xe => Ok(VE),
            0xf => Ok(VF),
            _ => Err(InvalidRegisterNumber(format!("Invalid Register ID: {value}")))
        }
    }
}

impl Index<VRegister> for [u8] {
    type Output = u8;

    fn index(&self, reg: VRegister) -> &Self::Output {
        self.get(reg as usize).unwrap()
    }
}

impl IndexMut<VRegister> for [u8] {
    fn index_mut(&mut self, reg: VRegister) -> &mut Self::Output {
        self.get_mut(reg as usize).unwrap()
    }
}

impl Display for VRegister {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{self:?}")
    }
}