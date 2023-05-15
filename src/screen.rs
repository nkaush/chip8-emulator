use std::fmt::{self, Display, Formatter};

pub const NROWS: usize = 32;
pub const NCOLS: usize = 64;

pub struct Screen {
    pixels: [[bool; NCOLS]; NROWS]
}

impl Screen {
    pub fn new() -> Self {
        Self { pixels: [[false; NCOLS]; NROWS] }
    }

    pub fn clear(&mut self) {
        self.pixels = [[false; NCOLS]; NROWS];
    }

    pub fn flip(&mut self, x: usize, y: usize) -> Option<bool> {
        if x >= NCOLS || y >= NROWS {
            return None;
        } else {
            let out = self.pixels[y][x];
            self.pixels[y][x] = !out;
            Some(out)
        }
    }

    pub fn show(&self) {
        print!("\x1B[2J\x1B[H{}", self)
    }
}

impl Display for Screen {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        writeln!(f, "┌{}┐", "─".repeat(NCOLS))?;
        for row in 0..NROWS {
            write!(f, "│")?;
            for col in 0..NCOLS {
                if self.pixels[row][col] {
                    write!(f, "█")?
                } else {
                    write!(f, " ")?
                }
            }

            writeln!(f, "│")?
        }

        writeln!(f, "└{}┘", "─".repeat(NCOLS))?;

        Ok(())
    }
}