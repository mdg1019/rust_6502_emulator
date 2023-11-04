use super::status_flags::StatusFlags;

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
}

impl ToString for Registers {
    fn to_string(&self) -> String {
        "PC   A  X  Y  SP P  NV-BDIZC\r\n".to_string() +
        &format!("{:04X} {:02X} {:02X} {:02X} {:02X} {:02X} {}\r\n", 
            self.pc, self.a, self.x, self.y, self.sp, 
            self.p.to_byte(), self.p.to_string())
    }
}
