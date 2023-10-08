pub mod registers;

use registers::Registers;

pub struct Cpu {
  pub registers: Registers,
}

impl Cpu {
  pub fn new() -> Cpu {
    Cpu {
      registers: Registers::new(),
    }
  }
}