const SIXTY_FOUR_K_BYTES: usize = 64 * 1024;

pub struct Memory {
  pub memory: [u8; SIXTY_FOUR_K_BYTES],
}

impl Memory {
  pub fn new() -> Memory {
    Memory {
      memory: [0x00u8; SIXTY_FOUR_K_BYTES],
    }
  }

  pub fn get_8_bit_value(&self, location: usize) -> u8 {
    self.memory[location]
  }

  pub fn set_8_bit_value(&mut self, location: usize, value: u8) {
    self.memory[location] = value;
  }
  
  pub fn get_16_bit_value(&self, location: usize) -> u16 {
    let lsb = self.memory[location];
    let msb = self.memory[location + 1];

    (msb as u16) << 8 | lsb as u16
  }

  pub fn set_16_bit_value(&mut self, location: usize, value: u16) {
    let lsb = (value as u16) & 0x00ff;
    let msb = (value as u16) >> 8;

    self.memory[location] = lsb as u8;
    self.memory[location + 1] = msb as u8;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_8_bit_value() {
    let mut memory = Memory::new();

    memory.memory[0] = 0xff;

    let value = memory.get_8_bit_value(0);

    assert_eq!(value, 0xff);
  }

  #[test]
  fn test_set_8_bit_value() {
    let mut memory = Memory::new();

    assert_eq!(memory.memory[0], 0x00);

    memory.set_8_bit_value(0, 0xff);

    assert_eq!(memory.memory[0], 0xff);
  }

  #[test]
  fn test_get_16_bit_value() {
    let mut memory = Memory::new();

    memory.memory[0] = 0x2c;
    memory.memory[1] = 0xfd;

    let value = memory.get_16_bit_value(0);
    
    assert_eq!(value, 0xfd2c)
  }

  #[test]
  fn test_set_16_bit_value() {
    let mut memory = Memory::new();

    assert_eq!(memory.memory[0], 0x00);
    assert_eq!(memory.memory[1], 0x00);

    memory.set_16_bit_value(0, 0x2cfd);

    assert_eq!(memory.memory[0], 0xfd);
    assert_eq!(memory.memory[1], 0x2c);
  }
 }