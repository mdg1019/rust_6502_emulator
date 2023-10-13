pub mod registers;
pub mod memory;
pub mod instruction;

use registers::Registers;
use memory::Memory;
use instruction::Instruction;
use instruction::AddressingMode;
use instruction::ExecutionReturnValues;

const RESET_VECTOR: usize = 0xfffc;

const ADC_INSTRUCTION: &str = "ADC";
const LDA_INSTRUCTION: &str = "LDA";
const SBC_INSTRUCTION: &str = "SBC";

const INSTRUCTION_SET: [Instruction; 10] = [
  Instruction {
    opcode: 0x69,
    mnemonic: ADC_INSTRUCTION,
    bytes: 2,
    clock_periods: 2,
    addressing_mode: AddressingMode::Immediate,
    execute: Cpu::adc_instruction,
  },
  Instruction {
    opcode: 0xA1,
    mnemonic: LDA_INSTRUCTION,
    bytes: 2,
    clock_periods: 6,
    addressing_mode: AddressingMode::PreIndexedIndirect,
    execute: Cpu::lda_instruction,
  },
  Instruction {
    opcode: 0xA5,
    mnemonic: LDA_INSTRUCTION,
    bytes: 2,
    clock_periods: 3,
    addressing_mode: AddressingMode::ZeroPageDirect,
    execute: Cpu::lda_instruction,
  },
  Instruction {
    opcode: 0xA9,
    mnemonic: LDA_INSTRUCTION,
    bytes: 2,
    clock_periods: 2,
    addressing_mode: AddressingMode::Immediate,
    execute: Cpu::lda_instruction,
  },
  Instruction {
    opcode: 0xAD,
    mnemonic: LDA_INSTRUCTION,
    bytes: 3,
    clock_periods: 4,
    addressing_mode: AddressingMode::Absolute,
    execute: Cpu::lda_instruction,
  },
  Instruction {
    opcode: 0xB1,
    mnemonic: LDA_INSTRUCTION,
    bytes: 2,
    clock_periods: 5,
    addressing_mode: AddressingMode::PostIndexedIndirect,
    execute: Cpu::lda_instruction,
  },
  Instruction {
    opcode: 0xB5,
    mnemonic: LDA_INSTRUCTION,
    bytes: 2,
    clock_periods: 4,
    addressing_mode: AddressingMode::ZeroPageX,
    execute: Cpu::lda_instruction,
  },
  Instruction {
    opcode: 0xB9,
    mnemonic: LDA_INSTRUCTION,
    bytes: 3,
    clock_periods: 4,
    addressing_mode: AddressingMode::AbsoluteY,
    execute: Cpu::lda_instruction,
  },
  Instruction {
    opcode: 0xBD,
    mnemonic: LDA_INSTRUCTION,
    bytes: 3,
    clock_periods: 4,
    addressing_mode: AddressingMode::AbsoluteX,
    execute: Cpu::lda_instruction,
  },
  Instruction {
    opcode: 0xE9,
    mnemonic: "SBC",
    bytes: 2,
    clock_periods: 2,
    addressing_mode: AddressingMode::Immediate,
    execute: Cpu::sbc_instruction,
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
      AddressingMode::ZeroPageX => format!("${:02X},X", self.memory.get_8_bit_value(location + 1)),
      AddressingMode::Absolute => format!("${:04X}", self.memory.get_16_bit_value(location + 1)),
      AddressingMode::AbsoluteX => format!("${:04X},X", self.memory.get_16_bit_value(location + 1)),
      AddressingMode::AbsoluteY => format!("${:04X},Y", self.memory.get_16_bit_value(location + 1)),
      AddressingMode::PreIndexedIndirect => format!("(${:02X},X)", self.memory.get_8_bit_value(location + 1)),
      AddressingMode::PostIndexedIndirect => format!("(${:02X}),Y", self.memory.get_8_bit_value(location + 1)),
    };

    let result = format!("{:04X} {:<8} {:<4} {}", 
      location, bytes, instruction.mnemonic, operand);

    Some(result)
  }

  fn get_instruction_for_opcode(&self, location: usize) -> Option<Instruction> {
    let opcode = self.memory.get_8_bit_value(location);

    INSTRUCTION_SET.into_iter().find(|i| i.opcode == opcode)
  }

  fn get_value(&self, instruction: Instruction) -> (u8, bool) {
    match instruction.addressing_mode {
      AddressingMode::Immediate => {
        (self.memory.get_8_bit_value((self.registers.pc + 1) as usize), false)
      },
      AddressingMode::ZeroPageDirect => {
        let zero_page_offset = self.memory.get_8_bit_value((self.registers.pc + 1) as usize);

        (self.memory.get_8_bit_value(zero_page_offset as usize), false)
      },
      AddressingMode::ZeroPageX => {
        let zero_page_offset = self.memory.get_8_bit_value((self.registers.pc + 1) as usize);

        (self.memory.get_8_bit_value(zero_page_offset as usize + self.registers.x as usize), false)
      },
      AddressingMode::Absolute => {
        let address = self.memory.get_16_bit_value((self.registers.pc + 1) as usize);

        (self.memory.get_8_bit_value(address as usize), false)
      },
      AddressingMode::AbsoluteX => {
        let address = self.memory.get_16_bit_value((self.registers.pc + 1) as usize);

        (self.memory.get_8_bit_value(address as usize + self.registers.x as usize), Cpu::crosses_boundary(address, self.registers.x))
      },
      AddressingMode::AbsoluteY => {
        let address = self.memory.get_16_bit_value((self.registers.pc + 1) as usize);

        (self.memory.get_8_bit_value(address as usize + self.registers.y as usize), Cpu::crosses_boundary(address, self.registers.y))
      },
      AddressingMode::PreIndexedIndirect => {
        let indirect_address = self.memory.get_8_bit_value((self.registers.pc + 1) as usize) as usize + self.registers.x as usize;
        let address = self.memory.get_16_bit_value(indirect_address);

        (self.memory.get_8_bit_value(address as usize), false)
      },
      AddressingMode::PostIndexedIndirect => {
        let indirect_address = self.memory.get_8_bit_value((self.registers.pc + 1) as usize) as usize;
        let address = self.memory.get_16_bit_value(indirect_address);

        (self.memory.get_8_bit_value(address as usize + self.registers.y as usize), Cpu::crosses_boundary(address, self.registers.y))
      },
    }
  }

  pub fn set_zero_flag(&mut self, value: u8) {
    self.registers.p.zero_flag = value == 0;
  }

  pub fn set_negative_flag(&mut self, value: u8) {
    self.registers.p.negative_flag = value & 0x80 != 0;
  }

  pub fn set_overflow_flag(&mut self, a: u8, b: u8, result: u8) {
    // Overflow occurs if both numbers have the same sign and 
    // the result has a different sign.

    // !(a ^ b) - 0x80 bit will be set if both signs are true.
    // (a ^ result) - 0x80 bit will be set if result has a different sign.

    // Based on a StackOverflow answer: https://stackoverflow.com/questions/29193303/6502-emulation-proper-way-to-implement-adc-and-sbc
    
    self.registers.p.overflow_flag =  (!(a ^ b) & (a ^ result) & 0x80) != 0;
  }

  pub fn set_carry_flag(&mut self, result: u16) {
    self.registers.p.carry_flag = result > 0xff;
  }

  pub fn crosses_boundary(address: u16, offset: u8) -> bool{
    address >> 8 != (address + offset as u16) >> 8
  }

  pub fn adc_instruction(&mut self, instruction: Instruction) -> Option<ExecutionReturnValues> {
    let (value, crossed_boundary) = self.get_value(instruction);

    let carry = match self.registers.p.carry_flag {
      true => 1u16,
      false => 0u16,
    };

    let mut result = self.registers.a as u16 + value as u16 + carry;

    if self.registers.p.decimal_flag {
      if (self.registers.a & 0x0f) + (value & 0x0F) + carry as u8 > 9 {
        result += 6;
      }

      if result > 0x99 {
        result += 96;
      }
    }

    self.set_zero_flag(result as u8);
    self.set_negative_flag(result as u8);
    self.set_overflow_flag(self.registers.a, value, result as u8);
    self.set_carry_flag(result);

    self.registers.a = result as u8;

    let clock_periods = match crossed_boundary {
      true => instruction.clock_periods + 1,
      false => instruction.clock_periods,
    };

    Some(ExecutionReturnValues { bytes: instruction.bytes, clock_periods: clock_periods })
  }

  pub fn sbc_instruction(&mut self, instruction: Instruction) -> Option<ExecutionReturnValues> {
    let (value, crossed_boundary) = self.get_value(instruction);
  
    let carry = match self.registers.p.carry_flag {
      true => 0u16,
      false => 1u16,
    };

    let mut result = (self.registers.a as u16).wrapping_sub(value as u16 + carry);

    if self.registers.p.decimal_flag {
      if (self.registers.a & 0x0f) < (value & 0x0f) + carry as u8 {
        result -= 6;
      }

      if result & 0xFF > 0x99 {
        result -= 96;
      }
    }

    self.set_zero_flag(result as u8);
    self.set_negative_flag(result as u8);
    self.set_overflow_flag(self.registers.a, !value, result as u8);
    self.registers.p.carry_flag = !((self.registers.a as u16) < (value as u16 + carry));

    self.registers.a = result as u8;

    let clock_periods = match crossed_boundary {
      true => instruction.clock_periods + 1,
      false => instruction.clock_periods,
    };

    Some(ExecutionReturnValues { bytes: instruction.bytes, clock_periods: clock_periods })
  }

  pub fn lda_instruction(&mut self, instruction: Instruction) -> Option<ExecutionReturnValues> {
    let (value, crossed_boundary) = self.get_value(instruction);

    self.registers.a = value;

    self.set_zero_flag(self.registers.a);
    self.set_negative_flag(self.registers.a);

    let clock_periods = match crossed_boundary {
      true => instruction.clock_periods + 1,
      false => instruction.clock_periods,
    };

    Some(ExecutionReturnValues { bytes: instruction.bytes, clock_periods: clock_periods })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_set_zero_flag_when_not_zero() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.p.zero_flag = true;

    cpu.set_zero_flag(0xff);

    assert!(!cpu.registers.p.zero_flag);
  }

  #[test]
  fn test_set_zero_flag_when_zero() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.p.zero_flag = false;

    cpu.set_zero_flag(0x00);

    assert!(cpu.registers.p.zero_flag);
  }

  #[test]
  fn test_set_overflow_flag_when_two_positives_results_in_a_negative() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.p.overflow_flag = false;
    
    cpu.set_overflow_flag(0x7f, 0x01, (0x7f + 0x01) as u16 as u8);

    assert!(cpu.registers.p.overflow_flag);
  }

  #[test]
  fn test_set_overflow_flag_when_two_positives_results_in_a_positive() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.p.overflow_flag = false;
    
    cpu.set_overflow_flag(0x7e, 0x01, (0x7e + 0x01) as u16 as u8);

    assert!(!cpu.registers.p.overflow_flag);
  }

  #[test]
  fn test_set_overflow_flag_when_two_negatives_results_in_a_positive() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.p.overflow_flag = false;
    
    cpu.set_overflow_flag(0x80, 0xff, (0x80 + 0xff) as u16 as u8);

    assert!(cpu.registers.p.overflow_flag);
  }

  #[test]
  fn test_set_overflow_flag_when_two_negatives_results_in_a_negative() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.p.overflow_flag = false;
    
    cpu.set_overflow_flag(0x81, 0xff, (0x81 + 0xff) as u16 as u8);

    assert!(!cpu.registers.p.overflow_flag);
  }

  #[test]
  fn test_set_carry_flag_when_no_carry() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.p.carry_flag = true;

    cpu.set_carry_flag(0x00ff);

    assert!(!cpu.registers.p.carry_flag);
  }

  #[test]
  fn test_set_carry_flag_when_carry() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.p.carry_flag = false;

    cpu.set_carry_flag(0x0100);

    assert!(cpu.registers.p.carry_flag);
  }

  #[test]
  fn test_crosses_boundary_not_crossed() {
    assert!(!Cpu::crosses_boundary(0x1ffe, 0x01));
  }

  #[test]
  fn test_crosses_boundary_crossed() {
    assert!(Cpu::crosses_boundary(0x1fff, 0x01));
  }

  #[test]
  fn test_69_adc_immediate_instruction() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.a = 0x99;
    cpu.registers.p.zero_flag = true;
    cpu.registers.p.negative_flag = true;
    cpu.registers.p.overflow_flag = false;
    cpu.registers.p.carry_flag = false;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x8000] = 0x69;
    cpu.memory.memory[0x8001] = 0x99;

    let option_return_values = cpu.execute_opcode();
    
    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0x32);
    assert!(!cpu.registers.p.zero_flag);
    assert!(!cpu.registers.p.negative_flag);
    assert!(cpu.registers.p.overflow_flag);
    assert!(cpu.registers.p.carry_flag);
    assert_eq!(return_values.bytes, 2);
    assert_eq!(return_values.clock_periods, 2);
  }

  #[test]
  fn test_69_adc_immediate_instruction_with_carry_set_adds_correctly() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.a = 0xff;
    cpu.registers.p.zero_flag = true;
    cpu.registers.p.negative_flag = true;
    cpu.registers.p.overflow_flag = false;
    cpu.registers.p.carry_flag = true;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x8000] = 0x69;
    cpu.memory.memory[0x8001] = 0x01;

    let option_return_values = cpu.execute_opcode();
    
    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0x01);
    assert!(!cpu.registers.p.zero_flag);
    assert!(!cpu.registers.p.negative_flag);
    assert!(!cpu.registers.p.overflow_flag);
    assert!(cpu.registers.p.carry_flag);
    assert_eq!(return_values.bytes, 2);
    assert_eq!(return_values.clock_periods, 2);
  }

  #[test]
  fn test_69_adc_immediate_instruction_should_set_carry() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.a = 0xff;
    cpu.registers.p.zero_flag = true;
    cpu.registers.p.negative_flag = true;
    cpu.registers.p.overflow_flag = false;
    cpu.registers.p.carry_flag = false;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x8000] = 0x69;
    cpu.memory.memory[0x8001] = 0x01;

    let option_return_values = cpu.execute_opcode();
    
    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.registers.p.zero_flag);
    assert!(!cpu.registers.p.negative_flag);
    assert!(!cpu.registers.p.overflow_flag);
    assert!(cpu.registers.p.carry_flag);
    assert_eq!(return_values.bytes, 2);
    assert_eq!(return_values.clock_periods, 2);
  }

  #[test]
  fn test_69_adc_immediate_instruction_should_not_set_carry() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.a = 0xfe;
    cpu.registers.p.zero_flag = true;
    cpu.registers.p.negative_flag = true;
    cpu.registers.p.overflow_flag = false;
    cpu.registers.p.carry_flag = false;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x8000] = 0x69;
    cpu.memory.memory[0x8001] = 0x01;

    let option_return_values = cpu.execute_opcode();
    
    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0xff);
    assert!(!cpu.registers.p.zero_flag);
    assert!(cpu.registers.p.negative_flag);
    assert!(!cpu.registers.p.overflow_flag);
    assert!(!cpu.registers.p.carry_flag);
    assert_eq!(return_values.bytes, 2);
    assert_eq!(return_values.clock_periods, 2);
  }

  #[test]
  fn test_69_adc_immediate_instruction_should_overflow_1() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.a = 0x7f; // 127d
    cpu.registers.p.zero_flag = true;
    cpu.registers.p.negative_flag = true;
    cpu.registers.p.overflow_flag = false;
    cpu.registers.p.carry_flag = false;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x8000] = 0x69;
    cpu.memory.memory[0x8001] = 0x01; // 127d + 1d = 128d, which is an overflow.

    let option_return_values = cpu.execute_opcode();
    
    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0x80);
    assert!(!cpu.registers.p.zero_flag);
    assert!(cpu.registers.p.negative_flag);
    assert!(cpu.registers.p.overflow_flag);
    assert!(!cpu.registers.p.carry_flag);
    assert_eq!(return_values.bytes, 2);
    assert_eq!(return_values.clock_periods, 2);
  }

  #[test]
  fn test_69_adc_immediate_instruction_should_not_overflow_1() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.a = 0x7e; // 126 decimal
    cpu.registers.p.zero_flag = true;
    cpu.registers.p.negative_flag = true;
    cpu.registers.p.overflow_flag = false;
    cpu.registers.p.carry_flag = false;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x8000] = 0x69;
    cpu.memory.memory[0x8001] = 0x01; // 126d + 1d = 127d, which is not an overflow.

    let option_return_values = cpu.execute_opcode();
    
    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0x7f);
    assert!(!cpu.registers.p.zero_flag);
    assert!(!cpu.registers.p.negative_flag);
    assert!(!cpu.registers.p.overflow_flag);
    assert!(!cpu.registers.p.carry_flag);
    assert_eq!(return_values.bytes, 2);
    assert_eq!(return_values.clock_periods, 2);
  }

  #[test]
  fn test_69_adc_immediate_instruction_should_overflow_2() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.a = 0x81; // -127
    cpu.registers.p.zero_flag = true;
    cpu.registers.p.negative_flag = true;
    cpu.registers.p.overflow_flag = false;
    cpu.registers.p.carry_flag = false;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x8000] = 0x69;
    cpu.memory.memory[0x8001] = 0xfe; // -127d + -2d = -129d, which is an overflow.

    let option_return_values = cpu.execute_opcode();  
    
    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0x7f);
    assert!(!cpu.registers.p.zero_flag);
    assert!(!cpu.registers.p.negative_flag);
    assert!(cpu.registers.p.overflow_flag);
    assert!(cpu.registers.p.carry_flag);
    assert_eq!(return_values.bytes, 2);
    assert_eq!(return_values.clock_periods, 2);
  }

  #[test]
  fn test_69_adc_immediate_instruction_should_not_overflow_2() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.a = 0x81; // -127 decimal
    cpu.registers.p.zero_flag = true;
    cpu.registers.p.negative_flag = true;
    cpu.registers.p.overflow_flag = false;
    cpu.registers.p.carry_flag = false;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x8000] = 0x69;
    cpu.memory.memory[0x8001] = 0xff; // -127d + -1d = -128d, which is not an overflow.

    let option_return_values = cpu.execute_opcode();
    
    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0x80);
    assert!(!cpu.registers.p.zero_flag);
    assert!(cpu.registers.p.negative_flag);
    assert!(!cpu.registers.p.overflow_flag);
    assert!(cpu.registers.p.carry_flag);
    assert_eq!(return_values.bytes, 2);
    assert_eq!(return_values.clock_periods, 2);
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

  #[test]
  fn test_b5_lda_zero_page_x_instruction() {
    let mut cpu: Cpu = Cpu::new(0x8000);

    cpu.registers.a = 0xff;
    cpu.registers.x = 0x02;
    cpu.registers.p.zero_flag = false;
    cpu.registers.p.negative_flag = true;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x32] = 0x00;
    cpu.memory.memory[0x8000] = 0xb5;
    cpu.memory.memory[0x8001] = 0x30;

    let option_return_values = cpu.execute_opcode();

    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0x00);
    assert!(cpu.registers.p.zero_flag);
    assert!(!cpu.registers.p.negative_flag);
    assert_eq!(return_values.bytes, 2);
    assert_eq!(return_values.clock_periods, 4);
  }

  #[test]
  fn test_ad_lda_absolute_instruction() {
    let mut cpu: Cpu = Cpu::new(0x8000);

    cpu.registers.a = 0x00;
    cpu.registers.p.zero_flag = true;
    cpu.registers.p.negative_flag = false;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x3000] = 0xff;
    cpu.memory.memory[0x8000] = 0xad;
    cpu.memory.memory[0x8001] = 0x00;
    cpu.memory.memory[0x8002] = 0x30;

    let option_return_values = cpu.execute_opcode();

    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0xff);
    assert!(!cpu.registers.p.zero_flag);
    assert!(cpu.registers.p.negative_flag);
    assert_eq!(return_values.bytes, 3);
    assert_eq!(return_values.clock_periods, 4);
  }

  #[test]
  fn test_ad_lda_absolute_x_instruction() {
    let mut cpu: Cpu = Cpu::new(0x8000);

    cpu.registers.a = 0x00;
    cpu.registers.x = 0x02;
    cpu.registers.p.zero_flag = true;
    cpu.registers.p.negative_flag = false;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x3002] = 0xff;
    cpu.memory.memory[0x8000] = 0xbd;
    cpu.memory.memory[0x8001] = 0x00;
    cpu.memory.memory[0x8002] = 0x30;

    let option_return_values = cpu.execute_opcode();

    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0xff);
    assert!(!cpu.registers.p.zero_flag);
    assert!(cpu.registers.p.negative_flag);
    assert_eq!(return_values.bytes, 3);
    assert_eq!(return_values.clock_periods, 4);
  }
  
  #[test]
  fn test_ad_lda_absolute_y_instruction() {
    let mut cpu: Cpu = Cpu::new(0x8000);

    cpu.registers.a = 0x00;
    cpu.registers.y = 0x02;
    cpu.registers.p.zero_flag = true;
    cpu.registers.p.negative_flag = false;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x3002] = 0xff;
    cpu.memory.memory[0x8000] = 0xb9;
    cpu.memory.memory[0x8001] = 0x00;
    cpu.memory.memory[0x8002] = 0x30;

    let option_return_values = cpu.execute_opcode();

    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0xff);
    assert!(!cpu.registers.p.zero_flag);
    assert!(cpu.registers.p.negative_flag);
    assert_eq!(return_values.bytes, 3);
    assert_eq!(return_values.clock_periods, 4);
  }
  
  #[test]
  fn test_ad_lda_pre_indexed_indirect_instruction() {
    let mut cpu: Cpu = Cpu::new(0x8000);

    cpu.registers.a = 0x00;
    cpu.registers.x = 0x05;
    cpu.registers.p.zero_flag = true;
    cpu.registers.p.negative_flag = false;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x0075] = 0x32;
    cpu.memory.memory[0x0076] = 0x30;
    cpu.memory.memory[0x3032] = 0xff;
    cpu.memory.memory[0x8000] = 0xa1;
    cpu.memory.memory[0x8001] = 0x70;

    let option_return_values = cpu.execute_opcode();

    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0xff);
    assert!(!cpu.registers.p.zero_flag);
    assert!(cpu.registers.p.negative_flag);
    assert_eq!(return_values.bytes, 2);
    assert_eq!(return_values.clock_periods, 6);
  }
  
  #[test]
  fn test_ad_lda_post_indexed_indirect_instruction() {
    let mut cpu: Cpu = Cpu::new(0x8000);

    cpu.registers.a = 0x00;
    cpu.registers.y = 0x10;
    cpu.registers.p.zero_flag = true;
    cpu.registers.p.negative_flag = false;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x0070] = 0x43;
    cpu.memory.memory[0x0071] = 0x35;
    cpu.memory.memory[0x3553] = 0xff;
    cpu.memory.memory[0x8000] = 0xb1;
    cpu.memory.memory[0x8001] = 0x70;

    let option_return_values = cpu.execute_opcode();

    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0xff);
    assert!(!cpu.registers.p.zero_flag);
    assert!(cpu.registers.p.negative_flag);
    assert_eq!(return_values.bytes, 2);
    assert_eq!(return_values.clock_periods, 5);
  }

  #[test]
  fn test_e9_sbc_immediate_instruction() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.a = 0x80;
    cpu.registers.p.zero_flag = true;
    cpu.registers.p.negative_flag = true;
    cpu.registers.p.overflow_flag = false;
    cpu.registers.p.carry_flag = true;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x8000] = 0xe9;
    cpu.memory.memory[0x8001] = 0x02;

    let option_return_values = cpu.execute_opcode();
    
    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0x7e);
    assert!(!cpu.registers.p.zero_flag);
    assert!(!cpu.registers.p.negative_flag);
    assert!(cpu.registers.p.overflow_flag);
    assert!(cpu.registers.p.carry_flag); // no borrow
    assert_eq!(return_values.bytes, 2);
    assert_eq!(return_values.clock_periods, 2);
  }

  #[test]
  fn test_e9_sbc_immediate_instruction_with_borrow() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.a = 0x80;
    cpu.registers.p.zero_flag = true;
    cpu.registers.p.negative_flag = true;
    cpu.registers.p.overflow_flag = false;
    cpu.registers.p.carry_flag = false;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x8000] = 0xe9;
    cpu.memory.memory[0x8001] = 0x02;

    let option_return_values = cpu.execute_opcode();
    
    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0x7d);
    assert!(!cpu.registers.p.zero_flag);
    assert!(!cpu.registers.p.negative_flag);
    assert!(cpu.registers.p.overflow_flag);
    assert!(cpu.registers.p.carry_flag); // no borrow
    assert_eq!(return_values.bytes, 2);
    assert_eq!(return_values.clock_periods, 2);
  }

  #[test]
  fn test_e9_sbc_immediate_instruction_sets_borrow() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.a = 0x04;
    cpu.registers.p.zero_flag = true;
    cpu.registers.p.negative_flag = true;
    cpu.registers.p.overflow_flag = false;
    cpu.registers.p.carry_flag = true;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x8000] = 0xe9;
    cpu.memory.memory[0x8001] = 0x0a;

    let option_return_values = cpu.execute_opcode();
    
    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0xfa);
    assert!(!cpu.registers.p.zero_flag);
    assert!(cpu.registers.p.negative_flag);
    assert!(!cpu.registers.p.overflow_flag);
    assert!(!cpu.registers.p.carry_flag); // no borrow
    assert_eq!(return_values.bytes, 2);
    assert_eq!(return_values.clock_periods, 2);
  }

  #[test]
  fn test_e9_sbc_immediate_instruction_should_overflow_1() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.a = 0x80; // -128d
    cpu.registers.p.zero_flag = true;
    cpu.registers.p.negative_flag = true;
    cpu.registers.p.overflow_flag = false;
    cpu.registers.p.carry_flag = true;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x8000] = 0xe9;
    cpu.memory.memory[0x8001] = 0x01; // -128d - 1d = -129d, which is an overflow

    let option_return_values = cpu.execute_opcode();
    
    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0x7f);
    assert!(!cpu.registers.p.zero_flag);
    assert!(!cpu.registers.p.negative_flag);
    assert!(cpu.registers.p.overflow_flag);
    assert!(cpu.registers.p.carry_flag); // no borrow
    assert_eq!(return_values.bytes, 2);
    assert_eq!(return_values.clock_periods, 2);
  }

  #[test]
  fn test_e9_sbc_immediate_instruction_should_not_overflow_1() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.a = 0x80; // -128d
    cpu.registers.p.zero_flag = true;
    cpu.registers.p.negative_flag = true;
    cpu.registers.p.overflow_flag = false;
    cpu.registers.p.carry_flag = true;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x8000] = 0xe9;
    cpu.memory.memory[0x8001] = 0x00; // -128d - 0d = -128d, which is not an overflow

    let option_return_values = cpu.execute_opcode();
    
    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0x80);
    assert!(!cpu.registers.p.zero_flag);
    assert!(cpu.registers.p.negative_flag);
    assert!(!cpu.registers.p.overflow_flag);
    assert!(cpu.registers.p.carry_flag); // no borrow
    assert_eq!(return_values.bytes, 2);
    assert_eq!(return_values.clock_periods, 2);
  }

  #[test]
  fn test_e9_sbc_immediate_instruction_should_overflow_2() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.a = 0x7f; // 128d
    cpu.registers.p.zero_flag = true;
    cpu.registers.p.negative_flag = true;
    cpu.registers.p.overflow_flag = false;
    cpu.registers.p.carry_flag = true;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x8000] = 0xe9;
    cpu.memory.memory[0x8001] = 0xff; // 127d - -1d = 128d, which is an overflow

    let option_return_values = cpu.execute_opcode();
    
    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0x80);
    assert!(!cpu.registers.p.zero_flag);
    assert!(cpu.registers.p.negative_flag);
    assert!(cpu.registers.p.overflow_flag);
    assert!(!cpu.registers.p.carry_flag); // borrow
    assert_eq!(return_values.bytes, 2);
    assert_eq!(return_values.clock_periods, 2);
  }

  #[test]
  fn test_e9_sbc_immediate_instruction_should_not_overflow_2() {
    let mut cpu: Cpu = Cpu::new(0x8000);
    cpu.registers.a = 0x7f; // 127d
    cpu.registers.p.zero_flag = true;
    cpu.registers.p.negative_flag = true;
    cpu.registers.p.overflow_flag = false;
    cpu.registers.p.carry_flag = true;
    cpu.registers.pc = 0x8000;

    cpu.memory.memory[0x8000] = 0xe9;
    cpu.memory.memory[0x8001] = 0x00; // 127d - 0d = 127d, which is not an overflow

    let option_return_values = cpu.execute_opcode();
    
    assert!(option_return_values.is_some());

    let return_values = option_return_values.unwrap();

    assert_eq!(cpu.registers.a, 0x7f);
    assert!(!cpu.registers.p.zero_flag);
    assert!(!cpu.registers.p.negative_flag);
    assert!(!cpu.registers.p.overflow_flag);
    assert!(cpu.registers.p.carry_flag); // no borrow
    assert_eq!(return_values.bytes, 2);
    assert_eq!(return_values.clock_periods, 2);
  }

  
}