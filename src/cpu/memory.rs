use std::fs::File;
use std::io::{self, Read};

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

  pub fn create_page_hexdump(&self, page: u8) -> String {
    let mut result = String::new();
    let mut address: usize = (page as usize) << 8;

    for row in 0..16 {
      let mut row_result = format!("{:04X} ", address);
      let mut hex_result = String::new();
      let mut ascii_result = String::new();

      for col in 0..16 {
        let byte = self.memory[address];

        hex_result = hex_result + &format!("{:02X} ", byte)[..];

       match byte {
          0..=31 => {
              ascii_result.push('.');
          }
          _ => {
              let byte_slice: &[u8] = &[byte];

              if let Ok(byte_str) = std::str::from_utf8(byte_slice) {
                  ascii_result.push_str(byte_str);
              } else {
                ascii_result.push('.');
              }
          }
      };

        address += 1;
      }

      row_result = row_result + &hex_result[..] + &ascii_result[..];


      result = result + &row_result[..] + "\r\n";
    }

    result
  }

  pub fn read_raw_file_into_memory(&mut self, file_path: &str, location: usize) -> io::Result<()> {
    let mut file = File::open(file_path)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut address = location;

    for &byte in &buffer {
      self.memory[address] = byte;

      address += 1;
    }

    Ok(())
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