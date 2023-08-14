use crate::{
    memory::{Memory, SegmentationFault}, address::{Address, InvalidAddress},
    register::{InvalidRegisterNumber, VRegister}, screen::{self, Screen},
    ticker::Ticker, isa::Instruction, 
};
use std::{
    sync::{Arc, atomic::{AtomicU8, Ordering}}, fs::File, 
    path::PathBuf, io::{self, Read, Write}, fmt::{Display, Formatter}
};
use rand::random;

const PC_INCREMENT: Address = Address(2);
const PC_START: Address = Address(0x200);
const NUM_REGISTERS: usize = 0x10;
const STACK_SIZE: usize = 0x10;

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
    i: Address,
    dt: Arc<AtomicU8>,
    st: Arc<AtomicU8>,
    pc: Address,
    sp: usize,
    stack: [Address; STACK_SIZE],
    pub memory: Memory,
    display: Screen,
    ticker: Ticker
}

impl Display for Cpu {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        for reg in 0..16 {
            let reg: VRegister = reg.try_into().unwrap();
            writeln!(f, "{reg:?} = {}", self.v[reg])?;
        }
        writeln!(f, "DT = {}", self.dt.load(Ordering::SeqCst))?;
        writeln!(f, "ST = {}", self.st.load(Ordering::SeqCst))?;
        writeln!(f, "PC = {}", self.pc)?;
        writeln!(f, "I  = {}", self.i)?;
        writeln!(f, "SP = {}", self.sp)?;
        writeln!(f, "STACK = {:?}", self.stack)?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum CpuError {
    StackOverflow,
    InfiniteLoop,
    InvalidAddress(String),
    InvalidRegister(String),
    SegmentationFault(Address),
    InvalidInstruction(u16),
    ProgramLoadError(io::Error)
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

impl From<SegmentationFault> for CpuError {
    fn from(e: SegmentationFault) -> Self {
        Self::SegmentationFault(e.0)
    }
}

impl From<io::Error> for CpuError {
    fn from(e: io::Error) -> Self {
        Self::ProgramLoadError(e)
    }
}

fn split_into_nibbles(i: u16) -> [u8; 4] {
    [
        ((i & 0xF000) >> 12) as u8, 
        ((i & 0x0F00) >> 8)  as u8, 
        ((i & 0x00F0) >> 4)  as u8, 
         (i & 0x000F)        as u8
    ]
}

impl Cpu {
    pub fn new(path: PathBuf) -> Result<Self, CpuError> {
        let mut program = Vec::new();
        let mut f = File::open(path)?;
        f.read_to_end(&mut program)?;

        let mut memory = Memory::new();
        memory.copy_to_offset(&SPRITES, SPRITES.len(), Address(0))?;
        memory.copy_to_offset(&program, program.len(), PC_START)?;

        let dt: Arc<AtomicU8> = Arc::new(0.into());
        let dtc: Arc<AtomicU8> = dt.clone();
        let ticker: Ticker = Ticker::new(move || {
            if dtc.load(Ordering::SeqCst) > 0 {
                dtc.fetch_sub(1, Ordering::SeqCst);
            }
        });
        
        Ok(Self {
            v: [0; NUM_REGISTERS],
            i: Address(0),
            dt,
            st: Arc::new(0.into()),
            pc: PC_START,
            sp: 0,
            stack: [Address(0); STACK_SIZE],
            memory,
            display: Screen::new(),
            ticker
        })
    }

    pub fn dump_core(&self) {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("core")
            .unwrap();

        f.write(format!("{}", self.memory).as_bytes()).unwrap();
    }

    pub fn fetch(&mut self) -> Result<u16, CpuError> {
        let instruction = self.memory
            .get_short(self.pc)?;

        self.pc += PC_INCREMENT;
        Ok(instruction)
    }

    pub fn decode(&self, instruction: u16) -> Result<Instruction, CpuError> {
        use Instruction::*;

        let nibbles: [u8; 4] = split_into_nibbles(instruction);
        let vx = nibbles[1].try_into();
        let vy = nibbles[2].try_into();
        let addr: Address = (instruction & Address::MASK).into();
        let lsb = (instruction & 0xFF) as u8;
        let lsn = (instruction & 0xF) as u8;

        match nibbles {
            [0x0, 0x0, 0xE, 0x0] => Ok(ClearScreen),
            [0x0, 0x0, 0xE, 0xE] => Ok(Return),
            [0x1, ..]            => Ok(Jump(addr)),
            [0x2, ..]            => Ok(Call(addr)),
            [0x3, ..]            => Ok(SkipIfEqualImm(vx?, lsb)),
            [0x4, ..]            => Ok(SkipIfNotEqualImm(vx?, lsb)),
            [0x5, .., 0x0]       => Ok(SkipIfEqual(vx?, vy?)),
            [0x6, ..]            => Ok(LoadImm(vx?, lsb)),
            [0x7, ..]            => Ok(AddImm(vx?, lsb)),
            [0x8, .., 0x0]       => Ok(Move(vx?, vy?)),
            [0x8, .., 0x1]       => Ok(Or(vx?, vy?)),
            [0x8, .., 0x2]       => Ok(And(vx?, vy?)),
            [0x8, .., 0x3]       => Ok(Xor(vx?, vy?)),
            [0x8, .., 0x4]       => Ok(Add(vx?, vy?)),
            [0x8, .., 0x5]       => Ok(Subtract(vx?, vy?)),
            [0x8, .., 0x6]       => Ok(ShiftRight(vx?)),
            [0x8, .., 0x7]       => Ok(SubtractN(vx?, vy?)),
            [0x8, .., 0xE]       => Ok(ShiftLeft(vx?)),
            [0x9, .., 0x0]       => Ok(SkipIfNotEqual(vx?, vy?)),
            [0xA, ..]            => Ok(LoadI(addr)),
            [0xB, ..]            => Ok(JumpOffset(addr)),
            [0xC, ..]            => Ok(AndRandom(vx?, lsb)),
            [0xD, ..]            => Ok(Draw(vx?, vy?, lsn)),
            [0xF, _, 0x0, 0x7]   => Ok(LoadDT(vx?)),
            [0xF, _, 0x1, 0x5]   => Ok(StoreDT(vx?)),
            [0xF, _, 0x1, 0x8]   => Ok(Nop),
            [0xF, _, 0x1, 0xE]   => Ok(AddI(vx?)),
            [0xF, _, 0x2, 0x9]   => Ok(LoadSprite(vx?)),
            [0xF, _, 0x3, 0x3]   => Ok(StoreBCD(vx?)),
            [0xF, _, 0x5, 0x5]   => Ok(Store(vx?)),
            [0xF, _, 0x6, 0x5]   => Ok(Load(vx?)),
            _ => Err(CpuError::InvalidInstruction(instruction))
        }
    }

    pub fn execute(&mut self, instruction: Instruction) -> Result<(), CpuError> {
        use Instruction::*;
        match instruction {
            Nop => (),
            ClearScreen => self.display.clear(),
            Return => {
                self.sp -= 1;
                self.pc = self.stack[self.sp];
            }
            Jump(addr) => {
                if self.pc - PC_INCREMENT == addr {
                    return Err(CpuError::InfiniteLoop)
                }
                self.pc = addr
            },
            JumpOffset(addr) => {
                self.pc = addr.offset(self.v[VRegister::V0] as u16);
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
            LoadImm(reg, imm) => {
                self.v[reg as usize] = imm
            },
            AddImm(reg, imm) => {
                self.v[reg] = self.v[reg].wrapping_add(imm)
            },
            Move(regx, regy) => {
                self.v[regx] = self.v[regy]
            },
            Or(regx, regy) => {
                self.v[regx] |= self.v[regy]
            },
            And(regx, regy) => {
                self.v[regx] &= self.v[regy]
            },
            Xor(regx, regy) => {
                self.v[regx] ^= self.v[regy]
            },
            AndRandom(reg, byte) => {
                self.v[reg] = byte & random::<u8>()
            },
            AddI(reg) => {
                self.i += self.v[reg].into()
            },
            Add(regx, regy) => {
                let vf = &mut false;
                (self.v[regx], *vf) = self.v[regx].overflowing_add(self.v[regy]);
                self.v[VRegister::VF] = *vf as u8;
            },
            Subtract(regx, regy) => {
                let (diff, overflow) = self.v[regx].overflowing_sub(self.v[regy]);
                self.v[VRegister::VF] = (!overflow) as u8;
                self.v[regx] = diff;
            },
            SubtractN(regx, regy) => {
                let (diff, overflow) = self.v[regy].overflowing_sub(self.v[regx]);
                self.v[VRegister::VF] = (!overflow) as u8;
                self.v[regx] = diff;
            },
            ShiftRight(regx) => {
                self.v[VRegister::VF] = self.v[regx] & 0x1;
                self.v[regx] >>= 1;
            },
            ShiftLeft(regx) => {
                self.v[VRegister::VF] = (self.v[regx] & 0x80) >> 7;
                self.v[regx] <<= 1;
            },
            LoadI(addr) => self.i = addr,
            LoadDT(reg) => {
                self.v[reg] = self.dt.load(Ordering::SeqCst);
            },
            StoreDT(reg) => {
                self.dt.store(self.v[reg], Ordering::SeqCst)
            },
            LoadSprite(reg) => {
                self.i = ((self.v[reg] & 0xF) * 5).into()
            },
            Load(reg) => {
                for r in 0u8..((reg as u8) + 1) {
                    let addr = self.i.offset(r as u16);
                    let reg: VRegister = r.try_into()?;
                    self.v[reg] = self.memory
                        .get_byte(addr)?;
                }
            },
            Store(reg) => {
                for r in 0u8..((reg as u8) + 1) {
                    let addr = self.i.offset(r as u16);
                    let reg: VRegister = r.try_into()?;
                    self.memory
                        .set_byte(addr, self.v[reg])?; 
                }
            },
            StoreBCD(reg) => {
                let val = self.v[reg];
                self.memory.set_byte(self.i, val / 100)?; 
                self.memory.set_byte(self.i.offset(1), (val / 10) % 10)?; 
                self.memory.set_byte(self.i.offset(2), val % 10)?;
            },
            Draw(regx, regy, n) => {
                let x = self.v[regx] & (screen::NCOLS as u8 - 1);
                let mut y = self.v[regy] & (screen::NROWS as u8 - 1);

                for offset in 0..(n.into()) {
                    let addr = self.i.offset(offset);
                    let data = self.memory.get_byte(addr)?;
                    let mut xx = x;

                    for i in (0u8..8).rev() {
                        if (1 << i) & data > 0 {
                            match self.display.flip(xx as usize, y as usize) {
                                Some(res) => {
                                    self.v[VRegister::VF] = res as u8
                                },
                                None => break
                            };
                        }
                        xx += 1;
                    }

                    y += 1;
                }

                self.display.show();
            }
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