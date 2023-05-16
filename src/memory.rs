use std::fmt::{Display, Formatter};
use crate::address::Address;

const MEMORY_SIZE: usize = 0x1000;

pub struct Memory {
    mem: [u8; MEMORY_SIZE],
}

impl Memory {
    pub fn new() -> Self {
        Self { mem: [0; MEMORY_SIZE] }
    }

    pub fn copy_to_offset(&mut self, data: &[u8], len: usize, offset: usize) {
        for i in 0..len {
            self.mem[offset + i] = data[i];
        }
    }

    pub fn set_byte(&mut self, address: Address, byte: u8) -> Option<u8> {
        let loc = self.mem.get_mut(address.0 as usize);
        if let Some(val) = loc {
            let out = *val;
            *val = byte;
            Some(out)
        } else {
            None
        }
    }

    pub fn get_byte(&self, address: Address) -> Option<u8> {
        self.mem.get(address.0 as usize).copied()
    }

    pub fn get_short(&self, address: Address) -> Option<u16> {
        let incr = address + Address(1);
        match (self.get_byte(address), self.get_byte(incr)) {
            (Some(msb), Some(lsb)) => Some(((msb as u16) << 8) | (lsb as u16)),
            _ => None
        }
    }
}

impl Display for Memory {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        const ROW_SIZE: usize = 16;

        for (idx, byte) in self.mem.iter().enumerate() {
            if idx % ROW_SIZE == 0 {
                if idx != 0 {
                    write!(f, "\n")?
                }

                write!(f, "{idx:08x}:")?
            }

            if idx % 2 == 0 {
                write!(f, " ")?
            }

            write!(f, "{byte:02x}")?
        }

        Ok(())
    }
}