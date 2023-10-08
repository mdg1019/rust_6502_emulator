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
}