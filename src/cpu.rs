pub mod registers;
pub mod memory;
pub mod instruction;

use registers::Registers;
use memory::Memory;
use instruction::Instruction;
use instruction::AddressingMode;
use instruction::ExecutionReturnValues;

const RESET_VECTOR: usize = 0xfffc;

const INSTRUCTION_SET: [Instruction; 2] = [
  Instruction {
    opcode: 0xA5,
    mnemonic: "LDA",
    bytes: 2,
    clock_periods: 3,
    addressing_mode: AddressingMode::ZeroPageDirect,
    execute: Cpu::lda_instruction,
  },
  Instruction {
    opcode: 0xA9,
    mnemonic: "LDA",
    bytes: 2,
    clock_periods: 2,
    addressing_mode: AddressingMode::Immediate,
    execute: Cpu::lda_instruction,
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

  pub fn execute_opcode(&mut self) -> Option<ExecutionReturnValues> {
    let instruction = self.get_instruction_for_opcode(self.registers.pc as usize)?;

    (instruction.execute)(self, instruction)
  }

  pub fn disassemble_opcode(&self, location: usize) -> Option<String> {
    let instruction = self.get_instruction_for_opcode(location)?;

    let mut bytes = String::new();

    for i in 0..instruction.bytes {
      bytes = format!("{} {:02X}", bytes, self.memory.get_8_bit_value(location + i as usize));
    }

    let operand = match instruction.addressing_mode {
      AddressingMode::ZeroPageDirect => format!("${:02X}", self.memory.get_8_bit_value(location + 1)),
      AddressingMode::Immediate => format!("#${:02X}", self.memory.get_8_bit_value(location + 1)),
    };

    let result = format!("{:04X} {:<8} {:<4} {}", 
      location, bytes, instruction.mnemonic, operand);

    Some(result)
  }

  fn get_instruction_for_opcode(&self, location: usize) -> Option<Instruction> {
    let opcode = self.memory.get_8_bit_value(location);

    INSTRUCTION_SET.into_iter().find(|i| i.opcode == opcode)
  }

  fn get_value(&self, instruction: Instruction) -> u8 {
    match instruction.addressing_mode {
      AddressingMode::Immediate => {
        self.memory.get_8_bit_value((self.registers.pc + 1) as usize)
      },
      AddressingMode::ZeroPageDirect => {
        let operand = self.memory.get_8_bit_value((self.registers.pc + 1) as usize);

        self.memory.get_8_bit_value(operand as usize)
      },
    }
  }

  pub fn set_zero_flag(&mut self) {
    self.registers.p.zero_flag = self.registers.a == 0;
  }

  pub fn set_negative_flag(&mut self) {
    self.registers.p.negative_flag = self.registers.a & 0x80 != 0;
  }

  pub fn lda_instruction(&mut self, instruction: Instruction) -> Option<ExecutionReturnValues> {
    let value = self.get_value(instruction);

    self.registers.a = value;

    self.set_zero_flag();
    self.set_negative_flag();

    Some(ExecutionReturnValues { bytes: instruction.bytes, clock_periods: instruction.clock_periods })
  }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_zero_flag_when_not_zero() {
      let mut cpu: Cpu = Cpu::new(0x8000);
      cpu.registers.p.zero_flag = true;
      cpu.registers.a = 0xff;

      cpu.set_zero_flag();

      assert!(!cpu.registers.p.zero_flag);
    }

    #[test]
    fn test_set_zero_flag_when_zero() {
      let mut cpu: Cpu = Cpu::new(0x8000);
      cpu.registers.p.zero_flag = false;
      cpu.registers.a = 0x00;

      cpu.set_zero_flag();

      assert!(cpu.registers.p.zero_flag);
    }

    #[test]
    fn test_a5_lda_zero_page_direct_instruction() {
      let mut cpu : Cpu = Cpu::new(0x8000);
      cpu.registers.a = 0x00;
      cpu.registers.p.zero_flag = true;
      cpu.registers.p.negative_flag = false;
      cpu.registers.pc = 0x8000;

      cpu.memory.memory[0x50] = 0xff;
      cpu.memory.memory[0x8000] = 0xa5;
      cpu.memory.memory[0x8001] = 0x50;

      let option_return_values = cpu.execute_opcode();
      
      assert!(option_return_values.is_some());

      let return_values = option_return_values.unwrap();

      assert_eq!(cpu.registers.a, 0xff);
      assert!(!cpu.registers.p.zero_flag);
      assert!(cpu.registers.p.negative_flag);
      assert_eq!(return_values.bytes, 2);
      assert_eq!(return_values.clock_periods, 3);
    }

    #[test]
    fn test_a9_lda_immediate_instruction() {
      let mut cpu : Cpu = Cpu::new(0x8000);
      cpu.registers.a = 0x00;
      cpu.registers.p.zero_flag = true;
      cpu.registers.p.negative_flag = false;
      cpu.registers.pc = 0x8000;

      cpu.memory.memory[0x8000] = 0xa9;
      cpu.memory.memory[0x8001] = 0xff;

      let option_return_values = cpu.execute_opcode();
      
      assert!(option_return_values.is_some());

      let return_values = option_return_values.unwrap();

      assert_eq!(cpu.registers.a, 0xff);
      assert!(!cpu.registers.p.zero_flag);
      assert!(cpu.registers.p.negative_flag);
      assert_eq!(return_values.bytes, 2);
      assert_eq!(return_values.clock_periods, 2);
    }


}