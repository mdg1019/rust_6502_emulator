use std::fs::File;
use std::io::Read;

const SIXTY_FOUR_K_BYTES: usize = 64 * 1024;

pub struct RomRegion {
    pub start: usize,
    pub end: usize,
}

pub struct Memory {
    pub contents: [u8; SIXTY_FOUR_K_BYTES],
    pub rom_regions: Vec<RomRegion>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            contents: [0x00u8; SIXTY_FOUR_K_BYTES],
            rom_regions: Vec::new(),
        }
    }

    pub fn is_in_rom_region(&mut self, address: usize) -> bool {
        for rom_region in &self.rom_regions {
            if address >= rom_region.start && address <= rom_region.end {
                return true;
            }
        }

        false
    }

    pub fn get_8_bit_value(&self, address: usize) -> u8 {
        self.contents[address]
    }

    pub fn set_8_bit_value(&mut self, address: usize, value: u8) {
        if !self.is_in_rom_region(address) {
            self.contents[address] = value;
        }
    }

    pub fn get_16_bit_value(&self, address: usize) -> u16 {
        let lsb = self.contents[address];
        let msb = self.contents[address + 1];

        (msb as u16) << 8 | lsb as u16
    }

    pub fn set_16_bit_value(&mut self, address: usize, value: u16) {
        if !self.is_in_rom_region(address) {
            let lsb = (value as u16) & 0x00ff;
            let msb = (value as u16) >> 8;

            self.contents[address] = lsb as u8;
            self.contents[address + 1] = msb as u8;
        }
    }

    pub fn create_page_hexdump(&self, page: u8) -> String {
        let mut result = String::new();
        let mut address: usize = (page as usize) << 8;

        for _ in 0..16 {
            let mut row_result = format!("{:04X} ", address);
            let mut hex_result = String::new();
            let mut ascii_result = String::new();

            for _ in 0..16 {
                let byte = self.contents[address];

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

    pub fn read_raw_file_into_memory(&mut self, file_path: &str, location: usize) -> usize {
        if let Ok(mut file) = File::open(file_path) {
            let mut buffer = Vec::new();

            if let Ok(length) = file.read_to_end(&mut buffer) {
                let mut address = location;

                for &byte in &buffer {
                    self.contents[address] = byte;

                    address += 1;
                }

                return length;
            }
        }

        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_8_bit_value() {
        let mut memory = Memory::new();

        memory.contents[0] = 0xff;

        let value = memory.get_8_bit_value(0);

        assert_eq!(value, 0xff);
    }

    #[test]
    fn test_set_8_bit_value() {
        let mut memory = Memory::new();

        assert_eq!(memory.contents[0], 0x00);

        memory.set_8_bit_value(0, 0xff);

        assert_eq!(memory.contents[0], 0xff);
    }

    #[test]
    fn test_get_16_bit_value() {
        let mut memory = Memory::new();

        memory.contents[0] = 0x2c;
        memory.contents[1] = 0xfd;

        let value = memory.get_16_bit_value(0);

        assert_eq!(value, 0xfd2c)
    }

    #[test]
    fn test_set_16_bit_value() {
        let mut memory = Memory::new();

        assert_eq!(memory.contents[0], 0x00);
        assert_eq!(memory.contents[1], 0x00);

        memory.set_16_bit_value(0, 0x2cfd);

        assert_eq!(memory.contents[0], 0xfd);
        assert_eq!(memory.contents[1], 0x2c);
    }

    #[test]
    fn test_is_in_rom_region() {
        let mut memory = Memory::new();

        assert!(!memory.is_in_rom_region(0x3000));

        memory.rom_regions.push(RomRegion {
            start: 0x3000,
            end: 0x3001,
        });

        assert!(!memory.is_in_rom_region(0x2FFF));
        assert!(memory.is_in_rom_region(0x3000));
        assert!(memory.is_in_rom_region(0x3001));
        assert!(!memory.is_in_rom_region(0x3002));
    }
}
