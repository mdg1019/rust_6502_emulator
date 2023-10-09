pub mod registers;
pub mod memory;

use registers::Registers;
use memory::Memory;

const RESET_VECTOR: usize = 0xfffc;

pub struct Cpu {
  pub registers: Registers,
  pub memory: Memory,
}

impl Cpu {
  pub fn new(reset_address: u16) -> Cpu {
    let mut cpu = Cpu {
      registers: Registers::new(),
      memory: Memory::new(),
    };

    cpu.memory.set_16_bit_value(RESET_VECTOR, reset_address);

    cpu
  }

  pub fn power_up(&mut self) {
    self.registers.pc = self.memory.get_16_bit_value(RESET_VECTOR);
  } 
}