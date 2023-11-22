use std::fmt::Display;

#[derive(Copy, Clone)]
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
    pub const CARRY_FLAG: u8 = 0x01;
    pub const ZERO_FLAG: u8 = 0x02;
    pub const INTERRUPT_FLAG: u8 = 0x04;
    pub const DECIMAL_FLAG: u8 = 0x08;
    pub const BREAK_FLAG: u8 = 0x10;
    pub const UNUSED_FLAG: u8 = 0x20;
    pub const OVERFLOW_FLAG: u8 = 0x40;
    pub const NEGATIVE_FLAG: u8 = 0x80;

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
            result |= StatusFlags::CARRY_FLAG;
        }

        if self.zero_flag {
            result |= StatusFlags::ZERO_FLAG;
        }

        if self.interrupt_disable_flag {
            result |= StatusFlags::INTERRUPT_FLAG;
        }

        if self.decimal_flag {
            result |= StatusFlags::DECIMAL_FLAG;
        }

        result |= StatusFlags::UNUSED_FLAG;

        if self.break_flag {
            result |= StatusFlags::BREAK_FLAG;
        }

        if self.overflow_flag {
            result |= StatusFlags::OVERFLOW_FLAG;
        }

        if self.negative_flag {
            result |= StatusFlags::NEGATIVE_FLAG;
        }

        result
    }

    pub fn from_byte(&mut self, byte: u8) {
        self.carry_flag = (byte & StatusFlags::CARRY_FLAG) != 0;
        self.zero_flag = (byte & StatusFlags::ZERO_FLAG) != 0;
        self.interrupt_disable_flag = (byte & StatusFlags::INTERRUPT_FLAG) != 0;
        self.decimal_flag = (byte & StatusFlags::DECIMAL_FLAG) != 0;
        self.break_flag = (byte & StatusFlags::BREAK_FLAG) != 0;
        self.overflow_flag = (byte & StatusFlags::OVERFLOW_FLAG) != 0;
        self.negative_flag = (byte & StatusFlags::NEGATIVE_FLAG) != 0;
    }
}

impl Display for StatusFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self.negative_flag {
            true => "1",
            false => "0",
        }
        .to_string()
            + match self.overflow_flag {
                true => "1",
                false => "0",
            }
            + "1"
            + match self.break_flag {
                true => "1",
                false => "0",
            }
            + match self.decimal_flag {
                true => "1",
                false => "0",
            }
            + match self.interrupt_disable_flag {
                true => "1",
                false => "0",
            }
            + match self.zero_flag {
                true => "1",
                false => "0",
            }
            + match self.carry_flag {
                true => "1",
                false => "0",
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_byte_for_no_carry() {
        let status_flags = StatusFlags::new();

        let value = status_flags.to_byte() & StatusFlags::CARRY_FLAG;

        assert_eq!(value, 0);
    }

    #[test]
    fn test_to_byte_for_carry() {
        let mut status_flags = StatusFlags::new();

        status_flags.carry_flag = true;

        let value = status_flags.to_byte() & StatusFlags::CARRY_FLAG;

        assert_eq!(value, StatusFlags::CARRY_FLAG);
    }

    #[test]
    fn test_to_byte_for_not_zero() {
        let status_flags = StatusFlags::new();

        let value = status_flags.to_byte() & StatusFlags::ZERO_FLAG;

        assert_eq!(value, 0);
    }

    #[test]
    fn test_to_byte_for_zero() {
        let mut status_flags = StatusFlags::new();

        status_flags.zero_flag = true;

        let value = status_flags.to_byte() & StatusFlags::ZERO_FLAG;

        assert_eq!(value, StatusFlags::ZERO_FLAG);
    }

    #[test]
    fn test_to_byte_for_interrupt_enabled() {
        let status_flags = StatusFlags::new();

        let value = status_flags.to_byte() & StatusFlags::INTERRUPT_FLAG;

        assert_eq!(value, 0);
    }

    #[test]
    fn test_to_byte_for_interrupt_disabled() {
        let mut status_flags = StatusFlags::new();

        status_flags.interrupt_disable_flag = true;

        let value = status_flags.to_byte() & StatusFlags::INTERRUPT_FLAG;

        assert_eq!(value, StatusFlags::INTERRUPT_FLAG);
    }

    #[test]
    fn test_to_byte_for_not_decimal_mode() {
        let status_flags = StatusFlags::new();

        let value = status_flags.to_byte() & StatusFlags::DECIMAL_FLAG;

        assert_eq!(value, 0);
    }

    #[test]
    fn test_to_byte_for_decimal_mode() {
        let mut status_flags = StatusFlags::new();

        status_flags.decimal_flag = true;

        let value = status_flags.to_byte() & StatusFlags::DECIMAL_FLAG;

        assert_eq!(value, StatusFlags::DECIMAL_FLAG);
    }

    #[test]
    fn test_to_byte_for_unused_set_to_1() {
        let status_flags = StatusFlags::new();

        let value = status_flags.to_byte() & StatusFlags::UNUSED_FLAG;

        assert_eq!(value, StatusFlags::UNUSED_FLAG);
    }

    #[test]
    fn test_to_byte_for_no_break() {
        let status_flags = StatusFlags::new();

        let value = status_flags.to_byte() & StatusFlags::BREAK_FLAG;

        assert_eq!(value, 0);
    }

    #[test]
    fn test_to_byte_for_break() {
        let mut status_flags = StatusFlags::new();

        status_flags.break_flag = true;

        let value = status_flags.to_byte() & StatusFlags::BREAK_FLAG;

        assert_eq!(value, StatusFlags::BREAK_FLAG);
    }

    #[test]
    fn test_to_byte_for_no_overflow() {
        let status_flags = StatusFlags::new();

        let value = status_flags.to_byte() & StatusFlags::OVERFLOW_FLAG;

        assert_eq!(value, 0);
    }

    #[test]
    fn test_to_byte_for_overflow() {
        let mut status_flags = StatusFlags::new();

        status_flags.overflow_flag = true;

        let value = status_flags.to_byte() & StatusFlags::OVERFLOW_FLAG;

        assert_eq!(value, StatusFlags::OVERFLOW_FLAG);
    }

    #[test]
    fn test_to_byte_for_not_negative() {
        let status_flags = StatusFlags::new();

        let value = status_flags.to_byte() & StatusFlags::NEGATIVE_FLAG;

        assert_eq!(value, 0);
    }

    #[test]
    fn test_to_byte_for_negative() {
        let mut status_flags = StatusFlags::new();

        status_flags.negative_flag = true;

        let value = status_flags.to_byte() & StatusFlags::NEGATIVE_FLAG;

        assert_eq!(value, StatusFlags::NEGATIVE_FLAG);
    }

    #[test]
    fn test_from_byte_for_no_carry() {
        let mut status_flags = StatusFlags::new();

        status_flags.carry_flag = true;
        status_flags.from_byte(!StatusFlags::CARRY_FLAG);

        assert!(!status_flags.carry_flag);
    }

    #[test]
    fn test_from_byte_for_carry() {
        let mut status_flags = StatusFlags::new();

        status_flags.from_byte(StatusFlags::CARRY_FLAG);

        assert!(status_flags.carry_flag);
    }

    #[test]
    fn test_from_byte_for_not_zero() {
        let mut status_flags = StatusFlags::new();

        status_flags.zero_flag = true;
        status_flags.from_byte(!StatusFlags::ZERO_FLAG);

        assert!(!status_flags.zero_flag);
    }

    #[test]
    fn test_from_byte_for_zero() {
        let mut status_flags = StatusFlags::new();

        status_flags.from_byte(StatusFlags::ZERO_FLAG);

        assert!(status_flags.zero_flag);
    }

    #[test]
    fn test_from_byte_for_interrupt_enabled() {
        let mut status_flags = StatusFlags::new();

        status_flags.interrupt_disable_flag = true;
        status_flags.from_byte(!StatusFlags::INTERRUPT_FLAG);

        assert!(!status_flags.interrupt_disable_flag);
    }

    #[test]
    fn test_from_byte_for_interrupt_disabled() {
        let mut status_flags = StatusFlags::new();

        status_flags.from_byte(StatusFlags::INTERRUPT_FLAG);

        assert!(status_flags.interrupt_disable_flag);
    }

    #[test]
    fn test_from_byte_for_not_decimal_mode() {
        let mut status_flags = StatusFlags::new();

        status_flags.decimal_flag = true;
        status_flags.from_byte(!StatusFlags::DECIMAL_FLAG);

        assert!(!status_flags.decimal_flag);
    }

    #[test]
    fn test_from_byte_for_decimal_mode() {
        let mut status_flags = StatusFlags::new();

        status_flags.from_byte(StatusFlags::DECIMAL_FLAG);

        assert!(status_flags.decimal_flag);
    }

    #[test]
    fn test_from_byte_for_no_break() {
        let mut status_flags = StatusFlags::new();

        status_flags.break_flag = true;
        status_flags.from_byte(!StatusFlags::BREAK_FLAG);

        assert!(!status_flags.break_flag);
    }

    #[test]
    fn test_from_byte_for_break() {
        let mut status_flags = StatusFlags::new();

        status_flags.from_byte(StatusFlags::BREAK_FLAG);

        assert!(status_flags.break_flag);
    }

    #[test]
    fn test_from_byte_for_no_overflow() {
        let mut status_flags = StatusFlags::new();

        status_flags.overflow_flag = true;
        status_flags.from_byte(!StatusFlags::OVERFLOW_FLAG);

        assert!(!status_flags.overflow_flag);
    }

    #[test]
    fn test_from_byte_for_overflow() {
        let mut status_flags = StatusFlags::new();

        status_flags.from_byte(StatusFlags::OVERFLOW_FLAG);

        assert!(status_flags.overflow_flag);
    }

    #[test]
    fn test_from_byte_for_not_negative() {
        let mut status_flags = StatusFlags::new();

        status_flags.negative_flag = true;
        status_flags.from_byte(!StatusFlags::NEGATIVE_FLAG);

        assert!(!status_flags.negative_flag);
    }

    #[test]
    fn test_from_byte_for_negative() {
        let mut status_flags = StatusFlags::new();

        status_flags.from_byte(StatusFlags::NEGATIVE_FLAG);

        assert!(status_flags.negative_flag);
    }

    #[test]
    fn test_to_string_for_not_negative() {
        let status_flags = StatusFlags::new();

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[0], '0' as u8);
    }

    #[test]
    fn test_to_string_for_negative() {
        let mut status_flags = StatusFlags::new();
        status_flags.negative_flag = true;

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[0], '1' as u8);
    }

    #[test]
    fn test_to_string_for_no_overflow() {
        let status_flags = StatusFlags::new();

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[1], '0' as u8);
    }

    #[test]
    fn test_to_string_for_overflow() {
        let mut status_flags = StatusFlags::new();
        status_flags.overflow_flag = true;

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[1], '1' as u8);
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

        assert_eq!(result.as_bytes()[3], '0' as u8);
    }

    #[test]
    fn test_to_string_for_break() {
        let mut status_flags = StatusFlags::new();
        status_flags.break_flag = true;

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[3], '1' as u8);
    }

    #[test]
    fn test_to_string_for_no_decimal_mode() {
        let status_flags = StatusFlags::new();

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[4], '0' as u8);
    }

    #[test]
    fn test_to_string_for_decimal_mode() {
        let mut status_flags = StatusFlags::new();
        status_flags.decimal_flag = true;

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[4], '1' as u8);
    }

    #[test]
    fn test_to_string_for_interrupt_enabled() {
        let status_flags = StatusFlags::new();

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[5], '0' as u8);
    }

    #[test]
    fn test_to_string_for_interrupt_disabled() {
        let mut status_flags = StatusFlags::new();
        status_flags.interrupt_disable_flag = true;

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[5], '1' as u8);
    }

    #[test]
    fn test_to_string_for_not_zero() {
        let status_flags = StatusFlags::new();

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[6], '0' as u8);
    }

    #[test]
    fn test_to_string_for_zero() {
        let mut status_flags = StatusFlags::new();
        status_flags.zero_flag = true;

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[6], '1' as u8);
    }

    #[test]
    fn test_to_string_for_no_carry() {
        let status_flags = StatusFlags::new();

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[7], '0' as u8);
    }

    #[test]
    fn test_to_string_for_carry() {
        let mut status_flags = StatusFlags::new();
        status_flags.carry_flag = true;

        let result = status_flags.to_string();

        assert_eq!(result.as_bytes()[7], '1' as u8);
    }
}
