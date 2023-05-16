use crate::{
    isa::Instruction, register::{InvalidRegisterNumber, VRegister}, 
    memory::Memory, address::{Address, InvalidAddress}, 
    screen::{self, Screen}, ticker::Ticker, 
};
use std::{
    sync::{Arc, atomic::{AtomicU8, Ordering}, mpsc::channel}, fs::File, 
    path::PathBuf, io::{self, Read}, fmt::{Display, Formatter}
};
use rand::random;

const STACK_SIZE: usize = 0x10;
const NUM_REGISTERS: usize = 0x10;
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
        writeln!(f, "I = {}", self.i)?;
        writeln!(f, "SP = {}", self.sp)?;
        writeln!(f, "STACK: {:?}", self.stack)?;

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
    InvalidInstruction(u16)
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
    [
        ((i & 0xF000) >> 12) as u8, 
        ((i & 0xF00) >> 8) as u8, 
        ((i & 0xF0) >> 4) as u8, 
        (i & 0xF) as u8
    ]
}

impl Cpu {
    pub fn new(path: PathBuf) -> Result<Self, io::Error> {
        let mut program = Vec::new();
        let mut f = File::open(path)?;
        f.read_to_end(&mut program)?;

        let mut memory = Memory::new();
        memory.copy_to_offset(&SPRITES, SPRITES.len(), 0);
        memory.copy_to_offset(&program, program.len(), PROGRAM_BASE);

        let (tx, rx) = channel();
        let dt: Arc<AtomicU8> = Arc::new(0.into());
        let dtc: Arc<AtomicU8> = dt.clone();
        let ticker: Ticker = Ticker::new(tx, move || {
            while let Ok(_) = rx.recv() {
                if dtc.load(Ordering::SeqCst) > 0 {
                    dtc.fetch_sub(1, Ordering::SeqCst);
                }
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

    pub fn fetch(&mut self) -> Result<u16, CpuError> {
        let instruction = self.memory
            .get_short(self.pc)
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
            [0x5, x, y, 0x0] => Ok(SkipIfEqual(x.try_into()?, y.try_into()?)),
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
            [0xA, n0, n1, n2] => Ok(LoadI([n0, n1, n2].try_into()?)),
            [0xB, n0, n1, n2] => Ok(JumpOffset([n0, n1, n2].try_into()?)),
            [0xC, x, k1, k2] => Ok(AndRandom(x.try_into()?, (k1 << 4) | k2)),
            [0xD, x, y, n] => Ok(Draw(x.try_into()?, y.try_into()?, n)),
            [0xF, x, 0x0, 0x7] => Ok(LoadDT(x.try_into()?)),
            [0xF, x, 0x1, 0x5] => Ok(StoreDT(x.try_into()?)),
            [0xF, _x, 0x1, 0x8] => Ok(Nop),
            [0xF, x, 0x1, 0xE] => Ok(AddI(x.try_into()?)),
            [0xF, x, 0x2, 0x9] => Ok(LoadSprite(x.try_into()?)),
            [0xF, x, 0x3, 0x3] => Ok(StoreBCD(x.try_into()?)),
            [0xF, x, 0x5, 0x5] => Ok(Store(x.try_into()?)),
            [0xF, x, 0x6, 0x5] => Ok(Load(x.try_into()?)),
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
                        .get_byte(addr)
                        .ok_or_else(|| CpuError::SegmentationFault(self.pc))?;
                }
            },
            Store(reg) => {
                for r in 0u8..((reg as u8) + 1) {
                    let addr = self.i.offset(r as u16);
                    let reg: VRegister = r.try_into()?;
                    self.memory
                        .set_byte(addr, self.v[reg])
                        .ok_or_else(|| CpuError::SegmentationFault(self.pc))?; 
                }
            },
            StoreBCD(reg) => {
                let val = self.v[reg];
                self.memory
                    .set_byte(self.i, val / 100)
                    .ok_or_else(|| CpuError::SegmentationFault(self.pc))?; 
                self.memory
                    .set_byte(self.i.offset(1), (val / 10) % 10)
                    .ok_or_else(|| CpuError::SegmentationFault(self.pc))?; 
                self.memory
                    .set_byte(self.i.offset(2), val % 10)
                    .ok_or_else(|| CpuError::SegmentationFault(self.pc))?;
            },
            Draw(regx, regy, n) => {
                let x = self.v[regx] & (screen::NCOLS as u8 - 1);
                let mut y = self.v[regy] & (screen::NROWS as u8 - 1);

                for offset in 0..(n.into()) {
                    let addr = self.i.offset(offset);
                    let data = match self.memory.get_byte(addr) {
                        Some(data) => data,
                        None => return Err(CpuError::SegmentationFault(addr))
                    };
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