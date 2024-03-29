use crate::{address::Address, register::VRegister};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Instruction {
    /// `00E0` - `CLS`: Clear the display.
    ClearScreen,
    /// `00EE` - `RET`: Return from a subroutine. The interpreter sets the 
    /// program counter to the address at the top of the stack, then subtracts 1 
    /// from the stack pointer.
    Return,
    /// `1nnn` - `JP addr`: Jump to location `nnn`. The interpreter sets the 
    /// program counter to `nnn`.
    Jump(Address),
    /// `2nnn` - `CALL addr`: Call subroutine at `nnn`. The interpreter 
    /// increments the stack pointer, then puts the current PC on the top of the 
    /// stack. The PC is then set to `nnn`.
    Call(Address),
    /// `3xkk` - `SE Vx, byte`: Skip next instruction if `Vx` = `kk`. The 
    /// interpreter compares register `Vx` to `kk`, and if they are equal, 
    /// increments the program counter by 2.
    SkipIfEqualImm(VRegister, u8),
    /// `4xkk` - `SNE Vx`, byte: Skip next instruction if `Vx` != `kk`. The 
    /// interpreter compares register `Vx` to `kk`, and if they are not equal, 
    /// increments the program counter by 2.
    SkipIfNotEqualImm(VRegister, u8),
    /// `5xy0` - `SE Vx, Vy`: Skip next instruction if `Vx` = `Vy`. The
    /// interpreter compares register `Vx` to register `Vy`, and if they are 
    /// equal, increments the program counter by 2.
    SkipIfEqual(VRegister, VRegister), 
    /// `6xkk` - `LD Vx, byte`: Set `Vx` = `kk`. The interpreter puts the value 
    /// `kk` into register `Vx`.
    LoadImm(VRegister, u8),
    /// `7xkk` - `ADD Vx, byte`: Set `Vx` = `Vx` + `kk`. Adds the value `kk` to 
    /// the value of register `Vx`, then stores the result in `Vx`.
    AddImm(VRegister, u8),
    /// `8xy0` - `LD Vx, Vy`: Set `Vx` = `Vy`. Stores the value of register `Vy` 
    /// in register `Vx`.
    Move(VRegister, VRegister), 
    /// `8xy1` - `OR Vx, Vy`: Set `Vx` = `Vx` OR `Vy`. Performs a bitwise OR on 
    /// the values of `Vx` and `Vy`, then stores the result in `Vx`. A bitwise 
    /// OR compares the corrseponding bits from two values, and if either bit is 
    /// 1, then the same bit in the result is also 1. Otherwise, it is 0.
    Or(VRegister, VRegister),
    /// `8xy2` - `AND Vx, Vy`: Set `Vx` = `Vx` AND `Vy`. Performs a bitwise AND
    /// on the values of `Vx` and `Vy`, then stores the result in `Vx`. A 
    /// bitwise AND compares the corrseponding bits from two values, and if both 
    /// bits are 1, then the same bit in the result is also 1. Otherwise, it's 0.
    And(VRegister, VRegister),
    /// `8xy3` - `XOR Vx, Vy`: Set `Vx` = `Vx` XOR `Vy`. Performs a bitwise 
    /// exclusive OR on the values of `Vx` and `Vy`, then stores the result in 
    /// `Vx`. An exclusive OR compares the corrseponding bits from two values, 
    /// and if the bits are not both the same, then the corresponding bit in the 
    /// result is set to 1. Otherwise, it is 0.
    Xor(VRegister, VRegister),
    /// `8xy4` - `ADD Vx, Vy`: Set `Vx` = `Vx` + `Vy`, set `VF` = `carry`. The 
    /// values of Vx and Vy are added together. If the result is greater than 8 
    /// bits (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits 
    /// of the result are kept, and stored in Vx.
    Add(VRegister, VRegister),
    /// `8xy5` - `SUB Vx, Vy`: Set `Vx` = `Vx` - `Vy`, set `VF` = `NOT borrow`. 
    /// If `Vx` > `Vy`, then `VF` is set to 1, otherwise 0. Then `Vy` is 
    /// subtracted from `Vx`, and the results stored in `Vx`.
    Subtract(VRegister, VRegister),
    /// `8x_6` - `SHR Vx`: Set `Vx` = `Vx` >> 1. If the least-significant bit of 
    /// `Vx` is 1, then VF is set to 1, otherwise 0. Then `Vx` is divided by 2.
    ShiftRight(VRegister),
    /// `8xy7` - `SUBN Vx, Vy`: Set `Vx` = `Vy` - `Vx`, set `VF` = `NOT borrow`.
    /// If `Vy` > `Vx`, then `VF` is set to 1, otherwise 0. Then `Vx` is 
    /// subtracted from `Vy`, and the results stored in `Vx`.
    SubtractN(VRegister, VRegister),
    /// `8xyE` - `SHL Vx`: Set `Vx` = `Vx` << 1. If the most-significant bit of 
    /// `Vx` is 1, then `VF` is set to 1, otherwise to 0. Then `Vx` is 
    /// multiplied by 2.
    ShiftLeft(VRegister),
    /// `9xy0` - `SNE Vx, Vy`: Skip next instruction if `Vx` != `Vy`. The values 
    /// of `Vx` and `Vy` are compared, and if they are not equal, the program 
    /// counter is increased by 2.
    SkipIfNotEqual(VRegister, VRegister),
    /// `Annn` - `LD I, addr`: Set `I` = `nnn`. The value of register `I` is set 
    /// to `nnn`.
    LoadI(Address),
    /// `Bnnn` - `JP V0, addr`: Jump to location `nnn` + `V0`. The program 
    /// counter is set to `nnn` plus the value of `V0`.
    JumpOffset(Address),
    /// `Cxkk` - `RND Vx, byte`: Set `Vx` = `random byte` AND `kk`. The 
    /// interpreter generates a random number from 0 to 255, which is then 
    /// ANDed with the value `kk`. The results are stored in `Vx`. See 
    /// instruction 8xy2 for more information on AND.
    AndRandom(VRegister, u8),
    /// `Dxyn` - `DRW Vx, Vy, nibble`: Display `n`-byte sprite starting at 
    /// memory location `I` at (`Vx`, `Vy`), set `VF` = `collision`. The 
    /// interpreter reads `n` bytes from memory, starting at the address stored 
    /// in `I`. These bytes are then displayed as sprites on screen at 
    /// coordinates (`Vx`, `Vy`). Sprites are XORed onto the existing screen. 
    /// If this causes any pixels to be erased, `VF` is set to 1, otherwise it 
    /// is set to 0. If the sprite is positioned so part of it is outside the 
    /// coordinates of the display, it wraps around to the opposite side of the 
    /// screen. See instruction 8xy3 for more information on XOR, and section 
    /// 2.4, Display, for more information on the Chip-8 screen and sprites.
    Draw(VRegister, VRegister, u8),
    /// `Fx07` - `LD Vx, DT`: Set `Vx` = delay timer value. The value of `DT` is 
    /// placed into `Vx`.
    LoadDT(VRegister),
    /// `Fx15` - `LD DT, Vx`: Set delay timer = `Vx`. `DT` is set equal to the 
    /// value of `Vx`.
    StoreDT(VRegister),
    /// `Fx1E` - `ADD I, Vx`: Set `I` = `I` + `Vx`. The values of `I` and `Vx` 
    /// are added, and the results are stored in `I`.
    AddI(VRegister),
    /// `Fx29` - `LD I, Vx`: Set `I` = location of sprite for digit `Vx`. The 
    /// value of I is set to the location for the hexadecimal sprite 
    /// corresponding to the value of `Vx`. See section 2.4, Display, for more 
    /// information on the Chip-8 hexadecimal font.
    LoadSprite(VRegister),
    /// `Fx33` - `LD I, Vx`: Store BCD representation of `Vx` in memory 
    /// locations `I`, `I+1`, and `I+2`. The interpreter takes the decimal value 
    /// of `Vx`, and places the hundreds digit in memory at location in `I`, the 
    /// tens digit at location `I+1`, and the ones digit at location `I+2`.
    StoreBCD(VRegister),
    /// `Fx55` - `LD [I], Vx`: Store registers `V0` through `Vx` in memory 
    /// starting at location `I`. The interpreter copies the values of registers 
    /// `V0` through `Vx` into memory, starting at the address in `I`.
    Store(VRegister),
    /// `Fx65` - `LD Vx, [I]`: Read registers `V0` through `Vx` from memory 
    /// starting at location `I`. The interpreter reads values from memory 
    /// starting at location `I` into registers `V0` through `Vx`.
    Load(VRegister),
    /// This instruction is not part of the official CHIP-8 ISA, but I have 
    /// added it regardless as a placeholder for instructions that are not yet 
    /// implemented by this interpreter. 
    Nop
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Instruction::*;

        match self {
            ClearScreen => write!(f, "CLS"),
            Return => write!(f, "RET"),
            Jump(addr) => write!(f, "JP {addr}"),
            Call(addr) => write!(f, "CALL {addr}"),
            SkipIfEqualImm(vx, b) => write!(f, "SE {vx}, {b}"),
            SkipIfNotEqualImm(vx, b) => write!(f, "SNE {vx}, {b}"),
            SkipIfEqual(vx, vy) => write!(f, "SE {vx}, {vy}"), 
            LoadImm(vx, b) => write!(f, "LD {vx}, {b}"),
            AddImm(vx, b) => write!(f, "ADD {vx}, {b}"),
            Move(vx, vy) => write!(f, "LD {vx}, {vy}"), 
            Or(vx, vy) => write!(f, "OR {vx}, {vy}"),
            And(vx, vy) => write!(f, "AND {vx}, {vy}"),
            Xor(vx, vy) => write!(f, "XOR {vx}, {vy}"),
            Add(vx, vy) => write!(f, "AND {vx}, {vy}"),
            Subtract(vx, vy) => write!(f, "SUB {vx}, {vy}"),
            ShiftRight(vx) => write!(f, "SHR {vx}"),
            SubtractN(vx, vy) => write!(f, "SUBN {vx}, {vy}"),
            ShiftLeft(vx) => write!(f, "SHL {vx}"),
            SkipIfNotEqual(vx, vy) => write!(f, "SNE {vx}, {vy}"),
            LoadI(addr) => write!(f, "LD I, {addr}"),
            JumpOffset(addr) => write!(f, "JP V0, {addr}"),
            AndRandom(vx, b) => write!(f, "RND {vx}, {b}"),
            Draw(vx, vy, b) => write!(f, "DRW {vx}, {vy}, {b}"),
            LoadDT(vx) => write!(f, "LD {vx}, DT"),
            StoreDT(vx) => write!(f, "LD DT, {vx}"),
            AddI(vx) => write!(f, "ADD I, {vx}"),
            LoadSprite(vx) => write!(f, "LD I, {vx}"),
            StoreBCD(vx) => write!(f, "LD I, {vx}"),
            Store(vx) => write!(f, "LD [I], {vx}"),
            Load(vx) => write!(f, "LD {vx}, [I]"),
            Nop => write!(f, "NOP")
        }
    }
}

// Ex9E - SKP Vx
// Skip next instruction if key with the value of Vx is pressed.
// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, PC is increased by 2.

// ExA1 - SKNP Vx
// Skip next instruction if key with the value of Vx is not pressed.
// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, PC is increased by 2.

// Fx0A - LD Vx, K
// Wait for a key press, store the value of the key in Vx.
// All execution stops until a key is pressed, then the value of that key is stored in Vx.

// Fx18 - LD ST, Vx
// Set sound timer = Vx.
// ST is set equal to the value of Vx.