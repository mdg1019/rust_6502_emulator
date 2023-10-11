use super::Cpu;

#[derive(Copy, Clone)]
pub enum AddressingMode {
  ZeroPageDirect,
}

#[derive(Debug)]
pub struct ExecutionReturnValues {
  pub bytes: u8,
  pub clock_periods: u8,
}

#[derive(Copy, Clone)]
pub struct Instruction {
  pub opcode: u8,
  pub mnemonic: &'static str,
  pub bytes: u8,
  pub clock_periods: u8,
  pub addressing_mode: AddressingMode,
  pub execute: fn(&mut Cpu, Instruction) -> Option<ExecutionReturnValues>,
}