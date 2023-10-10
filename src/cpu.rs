pub mod registers;
pub mod memory;
pub mod instruction;

use registers::Registers;
use memory::Memory;
use instruction::Instruction;
use instruction::AddressingMode;

const RESET_VECTOR: usize = 0xfffc;

const INSTRUCTION_SET: [Instruction; 1] = [
  Instruction {
    op_code: 0xA5,
    mnemonic: "LDA",
    clock_periods: 3,
    addressing_mode: AddressingMode::ZeroPageDirect,
  },
];

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