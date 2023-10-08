const CARRY_FLAG: u8 = 0x01;
const ZERO_FLAG: u8 = 0x02;
const INTERRUPT_DISABLE_FLAG: u8 = 0x04;
const DECIMAL_MODE_FLAG: u8 = 0x08;
const OVERFLOW_FLAG: u8 = 0x40;
const NEGATIVE_FLAG: u8 = 0x80;

pub struct StatusFlags {
  pub carry: bool,
  pub zero: bool,
  pub interrupt_disable: bool,
  pub decimal_mode: bool,
  pub overflow: bool,
  pub negative: bool,
}

impl StatusFlags {
    pub fn new() -> StatusFlags {
        StatusFlags {
            carry: false,
            zero: false,
            interrupt_disable: false,
            decimal_mode: false,
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
        if self.interrupt_disable {
            result |= INTERRUPT_DISABLE_FLAG;
        }
        if self.decimal_mode {
            result |= DECIMAL_MODE_FLAG;
        }
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
            interrupt_disable: (byte & INTERRUPT_DISABLE_FLAG) != 0,
            decimal_mode: (byte & DECIMAL_MODE_FLAG) != 0,
            overflow: (byte & OVERFLOW_FLAG) != 0,
            negative: (byte & NEGATIVE_FLAG) != 0,
        }
    }
}
