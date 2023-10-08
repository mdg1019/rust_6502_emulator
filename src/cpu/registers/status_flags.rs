const CARRY_FLAG: u8 = 0x01;
const ZERO_FLAG: u8 = 0x02;
const INTERRUPT_FLAG: u8 = 0x04;
const DECIMAL_FLAG: u8 = 0x08;
const BREAK_FLAG: u8 = 0x10;
const UNUSED_FLAG: u8 = 0x20;
const OVERFLOW_FLAG: u8 = 0x40;
const NEGATIVE_FLAG: u8 = 0x80;

pub struct StatusFlags {
    pub carry: bool,
    pub zero: bool,
    pub interrupt: bool,
    pub decimal: bool,
    pub break_flag: bool,
    unused: bool,
    pub overflow: bool,
    pub negative: bool,
}

impl StatusFlags {
    pub fn new() -> StatusFlags {
        StatusFlags {
            carry: false,
            zero: false,
            interrupt: false,
            decimal: false,
            break_flag: false,
            unused: true,
            overflow: false,
            negative: false,
        }
    }

    // Convert status flags to a single u8 value
    pub fn to_byte(&self) -> u8 {
        let mut result: u8 = 0;
        if self.carry {
            result |= CARRY_FLAG;
        }
        if self.zero {
            result |= ZERO_FLAG;
        }
        if self.interrupt {
            result |= INTERRUPT_FLAG;
        }
        if self.decimal {
            result |= DECIMAL_FLAG;
        }
        if self.break_flag {
            result |= BREAK_FLAG;
        }
        result != UNUSED_FLAG;
        if self.overflow {
            result |= OVERFLOW_FLAG;
        }
        if self.negative {
            result |= NEGATIVE_FLAG;
        }
        result
    }

    // Instantiate StatusFlags from a u8 value
    pub fn from_byte(byte: u8) -> StatusFlags {
        StatusFlags {
            carry: (byte & CARRY_FLAG) != 0,
            zero: (byte & ZERO_FLAG) != 0,
            interrupt: (byte & INTERRUPT_FLAG) != 0,
            decimal: (byte & DECIMAL_FLAG) != 0,
            break_flag: (byte & BREAK_FLAG) != 0,
            unused: true,
            overflow: (byte & OVERFLOW_FLAG) != 0,
            negative: (byte & NEGATIVE_FLAG) != 0,
        }
    }

    pub fn to_string(&self) -> String {
        match self.negative {
            true => "N",
            false => "-",
        }.to_string() +
        match self.overflow {
            true => "O",
            false => "-"
        } +
        "U" + 
        match self.decimal {
            true => "D",
            false => "-"
        } +
        match self.interrupt {
            true => "I",
            false => "-"
        } +
        match self.zero {
            true => "Z",
            false => "-"
        } +              
        match self.carry {
            true => "C",
            false => "-",
        }
    }
}
