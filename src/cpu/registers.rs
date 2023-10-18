mod status_flags;

use status_flags::StatusFlags;

pub struct Registers {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: StatusFlags,
    pub sp: u8,
    pub pc: u16,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0,
            x: 0,
            y: 0,
            p: StatusFlags::new(),
            sp: 0xff,
            pc: 0,
        }
    }

    pub fn to_string(&self) -> String {
        format!("A: {:02X}\r\n", self.a)
            + &format!("X: {:02X}\r\n", self.x)
            + &format!("Y: {:02X}\r\n", self.y)
            + &format!("P: {:02X} {}\r\n", self.p.to_byte(), self.p.to_string())
            + &format!("SP: {:02X}\r\n", self.sp)
            + &format!("PC: {:04X}", self.pc)
    }
}
