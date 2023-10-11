pub mod registers;
pub mod memory;
pub mod instruction;

use registers::Registers;
use memory::Memory;
use instruction::Instruction;
use instruction::AddressingMode;
use instruction::ExecutionReturnValues;

const RESET_VECTOR: usize = 0xfffc;

const INSTRUCTION_SET: [Instruction; 1] = [
  Instruction {
    opcode: 0xA5,
    mnemonic: "LDA",
    bytes: 2,
    clock_periods: 3,
    addressing_mode: AddressingMode::ZeroPageDirect,
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

  pub fn execute_opcode(&mut self, location: usize) -> Option<ExecutionReturnValues> {
    let instruction = self.get_instruction_for_opcode(location)?;

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
      _ => String::new(),
    };

    let result = format!("{:04X} {:<8} {:<4} {}", 
      location, bytes, instruction.mnemonic, operand);

    Some(result)
  }

  fn get_instruction_for_opcode(&self, location: usize) -> Option<Instruction> {
    let opcode = self.memory.get_8_bit_value(location);

    INSTRUCTION_SET.into_iter().find(|i| i.opcode == opcode)
  }

  fn get_value(cpu: &mut Cpu, instruction: Instruction) -> u8 {
    match instruction.addressing_mode {
      AddressingMode::ZeroPageDirect => {
        let operand = cpu.memory.get_8_bit_value((cpu.registers.pc + 1) as usize);

        cpu.memory.get_8_bit_value(operand as usize)
      },
      _ => panic!("Unknown Addressing mode"),
    }
  }

  fn check_for_zero_result(cpu: &mut Cpu) {
    if cpu.registers.a == 0 {
      cpu.registers.p.zero_flag = true;
    }
  }

  fn check_for_negative_result(cpu: &mut Cpu) {
    if cpu.registers.a & 0x80 != 0 {
      cpu.registers.p.negative_flag = true;
    }
  }

  fn lda_instruction(cpu: &mut Cpu, instruction: Instruction) -> Option<ExecutionReturnValues> {
    let value = Cpu::get_value(cpu, instruction);

    cpu.registers.a = value;

    Cpu::check_for_zero_result(cpu);
    Cpu::check_for_negative_result(cpu);

    Some(ExecutionReturnValues { bytes: instruction.bytes, clock_periods: instruction.clock_periods })
  }
}