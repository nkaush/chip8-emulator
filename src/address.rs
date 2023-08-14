use std::ops::{Add, AddAssign, Index, Sub};
use std::fmt::{Display, Formatter, Debug};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Address(pub u16);

impl Address {
    pub const MASK: u16 = 0xfff;

    pub fn offset(&self, off: u16) -> Address {
        Address(self.0 + off)
    }
}

pub struct InvalidAddress(pub String);

impl TryFrom<[u8; 3]> for Address {
    type Error = InvalidAddress;
    fn try_from(arr: [u8; 3]) -> Result<Self, Self::Error> { 
        if arr.iter().all(|x: &u8| x < &0x10) {
            let v = (arr[0] as u16) << 8 | (arr[1] as u16) << 4 | arr[2] as u16;
            Ok(Address(v))
        } else {
            Err(InvalidAddress(format!("Invalid Address: each value must be a nibble (i.e. value < 0x10): {arr:?}")))
        }
    }
}

impl From<u16> for Address {
    fn from(val: u16) -> Self {
        Address(val)
    }
}

impl From<u8> for Address {
    fn from(val: u8) -> Self {
        Address(val.into())
    }
}

impl Index<Address> for [u8] {
    type Output = u8;

    fn index(&self, addr: Address) -> &Self::Output {
        self.get(addr.0 as usize).unwrap()
    }
}

impl Add for Address {
    type Output = Address;
    fn add(self, decrement: Address) -> Self::Output { 
        Address(self.0 + decrement.0)
    }
}

impl Sub for Address {
    type Output = Address;
    fn sub(self, decrement: Address) -> Self::Output { 
        Address(self.0 - decrement.0)
    }
}

impl AddAssign for Address {
    fn add_assign(&mut self, increment: Address) { 
        self.0 += increment.0;
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> { 
        write!(f, "0x{:x}", self.0)
    }
}

impl Debug for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> { 
        write!(f, "{self}")
    }
}