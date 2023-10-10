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
    opcode: 0xA5,
    mnemonic: "LDA",
    bytes: 2,
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

  pub fn disassemble_opcode(&self, location: usize) -> Option<String> {
    let opcode = self.memory.get_8_bit_value(location);
    
    let instruction = match INSTRUCTION_SET.into_iter().find(|i| i.opcode == opcode) {
      Some(ins) => ins,
      None => return None,
    };

    let mut bytes = String::new();

    for i in 0..instruction.bytes {
      bytes = format!("{} {:02X}", bytes, self.memory.get_8_bit_value(location + i as usize));
    }

    let operand = match instruction.addressing_mode {
      AddressingMode::ZeroPageDirect => format!("${:02X}", self.memory.get_8_bit_value(location + 1)),
      _ => String::new(),
    };

    let result = format!("{:04X} {:<8} {:<4} {}", 
      location, bytes, instruction.mnemonic, operand);

    Some(result)

  }
}