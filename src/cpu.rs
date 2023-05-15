use crate::{
    isa::Instruction, register::{InvalidRegisterNumber, VRegister}, memory::Memory,
    address::{Address, InvalidAddress}, screen::{self, Screen}, 
};
use std::{fs::File, path::PathBuf, io::{self, Read}};

const STACK_SIZE: usize = 0x10;
const NUM_REGISTERS: usize = 0xF;
const PC_INCREMENT: Address = Address(2);

const PROGRAM_BASE: usize = 0x200;
const PC_START: Address = Address(PROGRAM_BASE as u16);

const SPRITES: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,
    0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xF0, 0x80, 0xF0,
    0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10,
    0xF0, 0x80, 0xF0, 0x10, 0xF0,
    0xF0, 0x80, 0xF0, 0x90, 0xF0,
    0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0,
    0xF0, 0x90, 0xF0, 0x10, 0xF0,
    0xF0, 0x90, 0xF0, 0x90, 0x90,
    0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0,
    0xE0, 0x90, 0x90, 0x90, 0xE0,
    0xF0, 0x80, 0xF0, 0x80, 0xF0,
    0xF0, 0x80, 0xF0, 0x80, 0x80
];


#[allow(dead_code)]
pub struct Cpu {
    v: [u8; NUM_REGISTERS],
    vf: bool,
    i: Address,
    dt: u8,
    st: u8,
    pc: Address,
    sp: usize,
    stack: [Address; STACK_SIZE],
    pub memory: Memory,
    display: Screen
}

#[derive(Debug)]
pub enum CpuError {
    StackOverflow,
    InfiniteLoop,
    InvalidAddress(String),
    InvalidRegister(String),
    SegmentationFault(Address)
}

impl From<InvalidAddress> for CpuError {
    fn from(e: InvalidAddress) -> Self {
        Self::InvalidAddress(e.0)
    }
}

impl From<InvalidRegisterNumber> for CpuError {
    fn from(e: InvalidRegisterNumber) -> Self {
        Self::InvalidRegister(e.0)
    }
}

fn split_into_nibbles(i: u16) -> [u8; 4] {
    [((i & 0xF000) >> 12) as u8, ((i & 0xF00) >> 8) as u8, ((i & 0xF0) >> 4) as u8, (i & 0xF) as u8]
}

impl Cpu {
    pub fn new(path: PathBuf) -> Result<Self, io::Error> {
        let mut program = Vec::new();
        let mut f = File::open(path)?;
        f.read_to_end(&mut program)?;

        let mut memory = Memory::new();
        memory.copy_to_offset(&SPRITES, 80, 0);
        memory.copy_to_offset(&program, program.len(), PROGRAM_BASE);

        Ok(Self {
            v: [0; NUM_REGISTERS],
            vf: false,
            i: Address(0),
            dt: 0,
            st: 0,
            pc: PC_START,
            sp: 0,
            stack: [Address(0); STACK_SIZE],
            memory,
            display: Screen::new()
        })
    }

    pub fn fetch(&mut self) -> Result<u16, CpuError> {
        let instruction = self.memory
            .get_short(self.pc.0 as usize)
            .ok_or_else(|| CpuError::SegmentationFault(self.pc));

        self.pc += PC_INCREMENT;
        instruction
    }

    pub fn decode(&self, instruction: u16) -> Result<Instruction, CpuError> {
        use Instruction::*;
        match split_into_nibbles(instruction) {
            [0x0, 0x0, 0xE, 0x0] => Ok(ClearScreen),
            [0x0, 0x0, 0xE, 0xE] => Ok(Return),
            [0x1, n0, n1, n2] => Ok(Jump([n0, n1, n2].try_into()?)),
            [0x2, n0, n1, n2] => Ok(Call([n0, n1, n2].try_into()?)),
            [0x3, x, k1, k2] => Ok(SkipIfEqualImm(x.try_into()?, (k1 << 4) | k2)),
            [0x4, x, k1, k2] => Ok(SkipIfNotEqualImm(x.try_into()?, (k1 << 4) | k2)),
            [0x5, x, y, 0] => Ok(SkipIfEqual(x.try_into()?, y.try_into()?)),
            [0x6, x, k1, k2] => Ok(LoadImm(x.try_into()?, (k1 << 4) | k2)),
            [0x7, x, k1, k2] => Ok(AddImm(x.try_into()?, (k1 << 4) | k2)),
            [0x8, x, y, 0x0] => Ok(Move(x.try_into()?, y.try_into()?)),
            [0x8, x, y, 0x1] => Ok(Or(x.try_into()?, y.try_into()?)),
            [0x8, x, y, 0x2] => Ok(And(x.try_into()?, y.try_into()?)),
            [0x8, x, y, 0x3] => Ok(Xor(x.try_into()?, y.try_into()?)),
            [0x8, x, y, 0x4] => Ok(Add(x.try_into()?, y.try_into()?)),
            [0x8, x, y, 0x5] => Ok(Subtract(x.try_into()?, y.try_into()?)),
            // [0x8, x, y, 0x6] => todo!(),
            [0x8, x, y, 0x7] => Ok(SubtractN(x.try_into()?, y.try_into()?)),
            // [0x8, x, y, 0x8] => todo!(),
            [0x9, x, y, 0] => Ok(SkipIfNotEqual(x.try_into()?, y.try_into()?)),
            [0xa, n0, n1, n2] => Ok(LoadI([n0, n1, n2].try_into()?)),
            [0xd, x, y, n] => Ok(Draw(x.try_into()?, y.try_into()?, n)),
            _ => unimplemented!()
        }
    }

    pub fn execute(&mut self, instruction: Instruction) -> Result<(), CpuError> {
        use Instruction::*;
        match instruction {
            ClearScreen => self.display.clear(),
            Return => {
                self.pc = self.stack[self.sp];
                self.sp -= 1;
            }
            Jump(addr) => {
                if self.pc - PC_INCREMENT == addr {
                    return Err(CpuError::InfiniteLoop)
                }
                self.pc = addr
            },
            JumpOffset(addr) => {
                self.pc = addr + self.v[VRegister::V0].into();
            }
            Call(addr) => {
                if self.sp >= STACK_SIZE {
                    return Err(CpuError::StackOverflow);
                }
                
                self.stack[self.sp] = self.pc;
                self.pc = addr;
                self.sp += 1;
            },
            SkipIfEqualImm(reg, imm) => {
                if self.v[reg] == imm {
                    self.pc += PC_INCREMENT;
                }
            },
            SkipIfNotEqualImm(reg, imm) => {
                if self.v[reg] != imm {
                    self.pc += PC_INCREMENT;
                }
            },
            SkipIfEqual(regx, regy) => {
                if self.v[regx] == self.v[regy] {
                    self.pc += PC_INCREMENT;
                }
            },
            SkipIfNotEqual(regx, regy) => {
                if self.v[regx] != self.v[regy] {
                    self.pc += PC_INCREMENT;
                }
            },
            LoadImm(reg, imm) => self.v[reg as usize] = imm,
            AddImm(reg, imm) => self.v[reg] = self.v[reg].wrapping_add(imm),
            Move(regx, regy) => self.v[regx] = self.v[regy],
            Or(regx, regy) => self.v[regx] |= self.v[regy],
            And(regx, regy) => self.v[regx] &= self.v[regy],
            Xor(regx, regy) => self.v[regx] ^= self.v[regy],
            Add(regx, regy) => {
                // let mut overflow = false;
                // (self.v[regx], *(&mut overflow)) = self.v[regx].overflowing_add(self.v[regy]);
                // self.v[0xF] = ...
                (self.v[regx], self.vf) = self.v[regx].overflowing_add(self.v[regy]);
            },
            Subtract(regx, regy) => {
                let (diff, overflow) = self.v[regx].overflowing_sub(self.v[regy]);
                self.v[regx] = diff;
                self.vf = !overflow;
            },
            SubtractN(regx, regy) => {
                let (diff, overflow) = self.v[regy].overflowing_sub(self.v[regx]);
                self.v[regx] = diff;
                self.vf = !overflow;
            },
            LoadI(addr) => self.i = addr,
            Draw(regx, regy, n) => {
                let x = self.v[regx] & (screen::NCOLS as u8 - 1);
                let mut y = self.v[regy] & (screen::NROWS as u8 - 1);

                for offset in 0..n {
                    let addr = self.i.offset(offset.into());
                    let data = match self.memory.get_byte(addr as usize) {
                        Some(data) => data,
                        None => return Err(CpuError::SegmentationFault(Address(addr)))
                    };
                    let mut xx = x;

                    for i in (0u8..8).rev() {
                        if (1 << i) & data > 0 {
                            match self.display.flip(xx as usize, y as usize) {
                                Some(res) => self.vf |= res,
                                None => break
                            };
                        }
                        xx += 1;
                    }

                    y += 1;
                }

                self.display.show();
            }
            // _ => ()
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_split_into_nibbles() {
        assert_eq!(split_into_nibbles(0x1234), [0x1, 0x2, 0x3, 0x4]);
        assert_eq!(split_into_nibbles(0xabcd), [0xa, 0xb, 0xc, 0xd]);
    }
}