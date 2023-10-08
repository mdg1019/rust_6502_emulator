
mod status_flags;

use status_flags::StatusFlags;

pub struct Registers {
pub a: u8,
pub x: u8,
pub y: u8,
pub status: StatusFlags,
pub sp: u8,
pub pc: u16,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0,
            x: 0,
            y: 0,
            status: StatusFlags::new(),
            sp: 0xFD, // Initialize stack pointer to its typical starting value
            pc: 0,    // Initialize program counter to 0
        }
    }

    pub fn to_string(&self) -> String {
        format!("A: {:02X}\r\n", self.a) +
        &format!("X: {:02X}\r\n", self.x) +
        &format!("Y: {:02X}\r\n", self.y) +
        &format!("Status: {:02X} {}\r\n", self.status.to_byte(), self.status.to_string()) +
        &format!("SP: {:02X}\r\n", self.sp) +
        &format!("PC: {:04X}", self.pc)  
    }
}