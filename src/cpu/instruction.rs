pub enum AddressingMode {
  ZeroPageDirect,
}

pub struct Instruction {
  pub op_code: u8,
  pub mnemonic: &'static str,
  pub clock_periods: u8,
  pub addressing_mode: AddressingMode,
}