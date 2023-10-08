pub mod registers;
pub mod memory;

use registers::Registers;
use memory::Memory;

pub struct Cpu {
  pub registers: Registers,
  pub memory: Memory,
}

impl Cpu {
  pub fn new() -> Cpu {
    Cpu {
      registers: Registers::new(),
      memory: Memory::new(),
    }
  }
}