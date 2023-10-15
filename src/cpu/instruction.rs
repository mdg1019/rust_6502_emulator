use super::Cpu;

#[derive(Copy, Clone)]
pub enum AddressingMode {
  Immediate,
  ZeroPage,
  ZeroPageX,
  Absolute,
  AbsoluteX,
  AbsoluteY,
  IndirectX, // (Zero-Page,X)
  IndirectY, // (Zero-Page),Y
}

#[derive(Debug)]
pub struct ExecutionReturnValues {
  pub bytes: u8,
  pub clock_periods: u8,
}

impl ExecutionReturnValues {
  pub fn new(instruction: Instruction, crossed_boundary: bool) -> ExecutionReturnValues {
    ExecutionReturnValues { 
      bytes: instruction.bytes, 
      clock_periods:  match crossed_boundary {
        true => instruction.clock_periods + 1,
        false => instruction.clock_periods,
      } 
    }
  }
}

#[derive(Copy, Clone)]
pub struct Instruction {
  pub opcode: u8,
  pub mnemonic: &'static str,
  pub bytes: u8,
  pub clock_periods: u8,
  pub addressing_mode: AddressingMode,
  pub execute: fn(&mut Cpu, Instruction) -> ExecutionReturnValues,
}