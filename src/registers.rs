use crate::status_flags::StatusFlags;


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
}