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

        result |= UNUSED_FLAG;

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

    pub fn from_byte(&mut self, byte: u8) {
        self.carry_flag = (byte & CARRY_FLAG) != 0;
        self.zero_flag = (byte & ZERO_FLAG) != 0;
        self.interrupt_disable_flag = (byte & INTERRUPT_FLAG) != 0;
        self.decimal_flag = (byte & DECIMAL_FLAG) != 0;
        self.break_flag = (byte & BREAK_FLAG) != 0;
        self.overflow_flag = (byte & OVERFLOW_FLAG) != 0;
        self.negative_flag = (byte & NEGATIVE_FLAG) != 0;
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
        match self.break_flag {
            true => "B",
            false => "-"
        } +
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_byte_for_no_carry() {
        let status_flags = StatusFlags::new();

        let value = status_flags.to_byte() & CARRY_FLAG;

        assert_eq!(value, 0);
    }

    #[test]
    fn test_to_byte_for_carry() {
        let mut status_flags = StatusFlags::new();

        status_flags.carry_flag = true;

        let value = status_flags.to_byte() & CARRY_FLAG;

        assert_eq!(value, CARRY_FLAG);
    }

    #[test]
    fn test_to_byte_for_not_zero() {
        let status_flags = StatusFlags::new();

        let value = status_flags.to_byte() & ZERO_FLAG;

        assert_eq!(value, 0);
    }

    #[test]
    fn test_to_byte_for_zero() {
        let mut status_flags = StatusFlags::new();

        status_flags.zero_flag = true;

        let value = status_flags.to_byte() & ZERO_FLAG;

        assert_eq!(value, ZERO_FLAG);
    }

    #[test]
    fn test_to_byte_for_interrupt_enabled() {
        let status_flags = StatusFlags::new();

        let value = status_flags.to_byte() & INTERRUPT_FLAG;

        assert_eq!(value, 0);
    }

    #[test]
    fn test_to_byte_for_interrupt_disabled() {
        let mut status_flags = StatusFlags::new();

        status_flags.interrupt_disable_flag = true;

        let value = status_flags.to_byte() & INTERRUPT_FLAG;

        assert_eq!(value, INTERRUPT_FLAG);
    }

    #[test]
    fn test_to_byte_for_not_decimal_mode() {
        let status_flags = StatusFlags::new();

        let value = status_flags.to_byte() & DECIMAL_FLAG;

        assert_eq!(value, 0);
    }

    #[test]
    fn test_to_byte_for_decimal_mode() {
        let mut status_flags = StatusFlags::new();

        status_flags.decimal_flag = true;

        let value = status_flags.to_byte() & DECIMAL_FLAG;

        assert_eq!(value, DECIMAL_FLAG);
    }

    #[test]
    fn test_to_byte_for_unused_set_to_1() {
        let status_flags = StatusFlags::new();

        let value = status_flags.to_byte() & UNUSED_FLAG;

        assert_eq!(value, UNUSED_FLAG);
    }

    #[test]
    fn test_to_byte_for_no_break() {
        let status_flags = StatusFlags::new();

        let value = status_flags.to_byte() & BREAK_FLAG;

        assert_eq!(value, 0);
    }

    #[test]
    fn test_to_byte_for_break() {
        let mut status_flags = StatusFlags::new();

        status_flags.break_flag = true;

        let value = status_flags.to_byte() & BREAK_FLAG;

        assert_eq!(value, BREAK_FLAG);
    }

    #[test]
    fn test_to_byte_for_no_overflow() {
        let status_flags = StatusFlags::new();

        let value = status_flags.to_byte() & OVERFLOW_FLAG;

        assert_eq!(value, 0);
    }

    #[test]
    fn test_to_byte_for_overflow() {
        let mut status_flags = StatusFlags::new();

        status_flags.overflow_flag = true;

        let value = status_flags.to_byte() & OVERFLOW_FLAG;

        assert_eq!(value, OVERFLOW_FLAG);
    }

    #[test]
    fn test_to_byte_for_not_negative() {
        let status_flags = StatusFlags::new();

        let value = status_flags.to_byte() & NEGATIVE_FLAG;

        assert_eq!(value, 0);
    }

    #[test]
    fn test_to_byte_for_negative() {
        let mut status_flags = StatusFlags::new();

        status_flags.negative_flag = true;

        let value = status_flags.to_byte() & NEGATIVE_FLAG;

        assert_eq!(value, NEGATIVE_FLAG);
    }

    #[test]
    fn test_from_byte_for_no_carry() {
        let mut status_flags = StatusFlags::new();

        status_flags.carry_flag = true;
        status_flags.from_byte(!CARRY_FLAG);

        assert!(!status_flags.carry_flag);  
    }

    #[test]
    fn test_from_byte_for_carry() {
        let mut status_flags = StatusFlags::new();

        status_flags.from_byte(CARRY_FLAG);

        assert!(status_flags.carry_flag);  
    }

    #[test]
    fn test_from_byte_for_not_zero() {
        let mut status_flags = StatusFlags::new();

        status_flags.zero_flag = true;
        status_flags.from_byte(!ZERO_FLAG);

        assert!(!status_flags.zero_flag);  
    }

    #[test]
    fn test_from_byte_for_zero() {
        let mut status_flags = StatusFlags::new();

        status_flags.from_byte(ZERO_FLAG);

        assert!(status_flags.zero_flag);  
    }

    #[test]
    fn test_from_byte_for_interrupt_enabled() {
        let mut status_flags = StatusFlags::new();

        status_flags.interrupt_disable_flag = true;
        status_flags.from_byte(!INTERRUPT_FLAG);

        assert!(!status_flags.interrupt_disable_flag);  
    }

    #[test]
    fn test_from_byte_for_interrupt_disabled() {
        let mut status_flags = StatusFlags::new();

        status_flags.from_byte(INTERRUPT_FLAG);

        assert!(status_flags.interrupt_disable_flag);  
    }

    #[test]
    fn test_from_byte_for_not_decimal_mode() {
        let mut status_flags = StatusFlags::new();

        status_flags.decimal_flag = true;
        status_flags.from_byte(!DECIMAL_FLAG);

        assert!(!status_flags.decimal_flag);  
    }

    #[test]
    fn test_from_byte_for_decimal_mode() {
        let mut status_flags = StatusFlags::new();

        status_flags.from_byte(DECIMAL_FLAG);

        assert!(status_flags.decimal_flag);  
    }

    #[test]
    fn test_from_byte_for_no_break() {
        let mut status_flags = StatusFlags::new();

        status_flags.break_flag = true;
        status_flags.from_byte(!BREAK_FLAG);

        assert!(!status_flags.break_flag);  
    }

    #[test]
    fn test_from_byte_for_break() {
        let mut status_flags = StatusFlags::new();

        status_flags.from_byte(BREAK_FLAG);

        assert!(status_flags.break_flag);  
    }

    #[test]
    fn test_from_byte_for_no_overflow() {
        let mut status_flags = StatusFlags::new();

        status_flags.overflow_flag = true;
        status_flags.from_byte(!OVERFLOW_FLAG);

        assert!(!status_flags.overflow_flag);  
    }

    #[test]
    fn test_from_byte_for_overflow() {
        let mut status_flags = StatusFlags::new();

        status_flags.from_byte(OVERFLOW_FLAG);

        assert!(status_flags.overflow_flag);  
    }

    #[test]
    fn test_from_byte_for_not_negative() {
        let mut status_flags = StatusFlags::new();

        status_flags.negative_flag = true;
        status_flags.from_byte(!NEGATIVE_FLAG);

        assert!(!status_flags.negative_flag);  
    }

    #[test]
    fn test_from_byte_for_negative() {
        let mut status_flags = StatusFlags::new();

        status_flags.from_byte(NEGATIVE_FLAG);

        assert!(status_flags.negative_flag);  
    }

    #[test]
    fn test_to_string_for_not_negative() {
        let status_flags = StatusFlags::new();

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[0], '-' as u8);
    }

    #[test]
    fn test_to_string_for_negative() {
        let mut status_flags = StatusFlags::new();
        status_flags.negative_flag = true;

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[0], 'N' as u8);
    }

    #[test]
    fn test_to_string_for_no_overflow() {
        let status_flags = StatusFlags::new();

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[1], '-' as u8);
    }

    #[test]
    fn test_to_string_for_overflow() {
        let mut status_flags = StatusFlags::new();
        status_flags.overflow_flag = true;

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[1], 'O' as u8);
    }

    #[test]
    fn test_to_string_for_unused_set_to_1() {
        let status_flags = StatusFlags::new();

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[2], '1' as u8);
    }

    #[test]
    fn test_to_string_for_no_break() {
        let status_flags = StatusFlags::new();

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[3], '-' as u8);
    }

    #[test]
    fn test_to_string_for_break() {
        let mut status_flags = StatusFlags::new();
        status_flags.break_flag = true;

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[3], 'B' as u8);
    }

    #[test]
    fn test_to_string_for_no_decimal_mode() {
        let status_flags = StatusFlags::new();

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[4], '-' as u8);
    }

    #[test]
    fn test_to_string_for_decimal_mode() {
        let mut status_flags = StatusFlags::new();
        status_flags.decimal_flag = true;

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[4], 'D' as u8);
    }

    #[test]
    fn test_to_string_for_interrupt_enabled() {
        let status_flags = StatusFlags::new();

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[5], '-' as u8);
    }

    #[test]
    fn test_to_string_for_interrupt_disabled() {
        let mut status_flags = StatusFlags::new();
        status_flags.interrupt_disable_flag = true;

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[5], 'I' as u8);
    }

    #[test]
    fn test_to_string_for_not_zero() {
        let status_flags = StatusFlags::new();

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[6], '-' as u8);
    }

    #[test]
    fn test_to_string_for_zero() {
        let mut status_flags = StatusFlags::new();
        status_flags.zero_flag = true;

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[6], 'Z' as u8);
    }

    #[test]
    fn test_to_string_for_no_carry() {
        let status_flags = StatusFlags::new();

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[7], '-' as u8);
    }

    #[test]
    fn test_to_string_for_carry() {
        let mut status_flags = StatusFlags::new();
        status_flags.carry_flag = true;

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[7], 'C' as u8);
    }
}