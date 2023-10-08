const CARRY_FLAG: u8 = 0x01;
const ZERO_FLAG: u8 = 0x02;
const INTERRUPT_FLAG: u8 = 0x04;
const DECIMAL_FLAG: u8 = 0x08;
const BREAK_FLAG: u8 = 0x10;
const UNUSED_FLAG: u8 = 0x20;
const OVERFLOW_FLAG: u8 = 0x40;
const NEGATIVE_FLAG: u8 = 0x80;

pub struct StatusFlags {
    pub carry_flag: bool,
    pub zero_flag: bool,
    pub interrupt_disable_flag: bool,
    pub decimal_flag: bool,
    pub break_flag: bool,
    pub overflow_flag: bool,
    pub negative_flag: bool,
}

impl StatusFlags {
    pub fn new() -> StatusFlags {
        StatusFlags {
            carry_flag: false,
            zero_flag: false,
            interrupt_disable_flag: false,
            decimal_flag: false,
            break_flag: false,
            overflow_flag: false,
            negative_flag: false,
        }
    }

    pub fn to_byte(&self) -> u8 {
        let mut result: u8 = 0;

        if self.carry_flag {
            result |= CARRY_FLAG;
        }

        if self.zero_flag {
            result |= ZERO_FLAG;
        }

        if self.interrupt_disable_flag {
            result |= INTERRUPT_FLAG;
        }

        if self.decimal_flag {
            result |= DECIMAL_FLAG;
        }
        if self.break_flag {
            result |= BREAK_FLAG;
        }

        if self.overflow_flag {
            result |= OVERFLOW_FLAG;
        }

        if self.negative_flag {
            result |= NEGATIVE_FLAG;
        }

        result
    }

    pub fn from_byte(byte: u8) -> StatusFlags {
        StatusFlags {
            carry_flag: (byte & CARRY_FLAG) != 0,
            zero_flag: (byte & ZERO_FLAG) != 0,
            interrupt_disable_flag: (byte & INTERRUPT_FLAG) != 0,
            decimal_flag: (byte & DECIMAL_FLAG) != 0,
            break_flag: (byte & BREAK_FLAG) != 0,
            overflow_flag: (byte & OVERFLOW_FLAG) != 0,
            negative_flag: (byte & NEGATIVE_FLAG) != 0,
        }
    }

    pub fn to_string(&self) -> String {
        match self.negative_flag {
            true => "N",
            false => "-",
        }.to_string() +
        match self.overflow_flag {
            true => "O",
            false => "-"
        } +
        "1" + 
        match self.decimal_flag {
            true => "D",
            false => "-"
        } +
        match self.interrupt_disable_flag {
            true => "I",
            false => "-"
        } +
        match self.zero_flag {
            true => "Z",
            false => "-"
        } +              
        match self.carry_flag {
            true => "C",
            false => "-",
        }
    }
}