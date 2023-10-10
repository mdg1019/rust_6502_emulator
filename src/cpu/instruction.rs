pub enum AddressingMode {
  ZeroPageDirect,
}

pub struct Instruction {
  pub opcode: u8,
  pub mnemonic: &'static str,
  pub bytes: u8,
  pub clock_periods: u8,
  pub addressing_mode: AddressingMode,
}