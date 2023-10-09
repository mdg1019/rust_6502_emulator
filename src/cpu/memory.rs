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

  pub fn set_8_bit_value(&mut self, location: usize, value: u8) {
    self.memory[location] = value;
  }

  pub fn get_8_bit_value(&self, location: usize) -> u8 {
    self.memory[location]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_set_8_bit_value() {
    let mut memory = Memory::new();

    assert_eq!(memory.memory[0], 0x00);

    memory.set_8_bit_value(0, 0xff);

    assert_eq!(memory.memory[0], 0xff);
  }

  #[test]
  fn test_get_8_bit_value() {
    let mut memory = Memory::new();

    assert_eq!(memory.memory[0], 0x00);

    memory.memory[0] = 0xff;

    let value = memory.get_8_bit_value(0);

    assert_eq!(value, 0xff);
  }
 }