pub mod instruction;
pub mod memory;
pub mod registers;
pub mod status_flags;

use instruction::AddressingMode;
use instruction::ExecutionReturnValues;
use instruction::Instruction;
use memory::Memory;
use registers::Registers;

const NMI_VECTOR: usize = 0xfffa;
const RESET_VECTOR: usize = 0xfffc;
const IRQ_BRK_VECTOR: usize = 0xfffe;
const STACK_BASE_ADDRESS: usize = 0x0100;

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
        self.registers.p.interrupt_disable_flag = true;
        self.registers.p.break_flag = false;
        self.registers.sp = 0xff;
        self.registers.pc = self.memory.get_16_bit_value(RESET_VECTOR);
    }

    pub fn execute_opcode(&mut self) -> Option<ExecutionReturnValues> {
        let instruction = self.get_instruction_for_opcode(self.registers.pc as usize)?;

        Some((instruction.execute)(self, instruction))
    }

    pub fn disassemble_opcode(&self, location: usize) -> Option<(String, u8)> {
        let instruction = self.get_instruction_for_opcode(location)?;

        let mut bytes = String::new();

        for i in 0..instruction.bytes {
            bytes = format!(
                "{} {:02X}",
                bytes,
                self.memory.get_8_bit_value(location + i as usize)
            );
        }

        let operand = match instruction.addressing_mode {
            AddressingMode::Accumulator => "A".to_string(),
            AddressingMode::Implied => String::new(),
            AddressingMode::Relative => {
                let offset = self.memory.get_8_bit_value(location + 1);
                let relative_address = Cpu::calculate_address_from_relative_offset((location + 2) as u16, offset);
                format!("${:04X}", relative_address)
            }
            AddressingMode::ZeroPage => {
                format!("${:02X}", self.memory.get_8_bit_value(location + 1))
            }
            AddressingMode::Immediate => {
                format!("#${:02X}", self.memory.get_8_bit_value(location + 1))
            }
            AddressingMode::ZeroPageX => {
                format!("${:02X},X", self.memory.get_8_bit_value(location + 1))
            }
            AddressingMode::ZeroPageY => {
                format!("${:02X},Y", self.memory.get_8_bit_value(location + 1))
            }
            AddressingMode::Absolute => {
                format!("${:04X}", self.memory.get_16_bit_value(location + 1))
            }
            AddressingMode::AbsoluteX => {
                format!("${:04X},X", self.memory.get_16_bit_value(location + 1))
            }
            AddressingMode::AbsoluteY => {
                format!("${:04X},Y", self.memory.get_16_bit_value(location + 1))
            }
            AddressingMode::Indirect => {
                format!("(${:04X})", self.memory.get_16_bit_value(location + 1) as usize)
            }
            AddressingMode::IndirectX => {
                format!("(${:02X},X)", self.memory.get_8_bit_value(location + 1))
            }
            AddressingMode::IndirectY => {
                format!("(${:02X}),Y", self.memory.get_8_bit_value(location + 1))
            }
        };

        let line = format!(
            "{:04X} {:<8} {:<4} {}",
            location, bytes, instruction.mnemonic, operand
        );

        Some((line, instruction.bytes))
    }

    fn branch(&mut self, instruction: Instruction, pred: bool) -> ExecutionReturnValues {
        if !pred {
            self.registers.pc += instruction.bytes as u16;
            return ExecutionReturnValues::new(instruction, false);
        }

        let old_pc = self.registers.pc;

        let offset = self.memory.contents[(self.registers.pc + 1) as usize];

        let relative_address = Cpu::calculate_address_from_relative_offset(self.registers.pc + 2, offset);

        self.registers.pc = relative_address;

        ExecutionReturnValues::new(
            instruction,
            Cpu::crosses_boundary_by_two_addresses(old_pc, relative_address)
        )
    }

    fn get_instruction_for_opcode(&self, location: usize) -> Option<Instruction> {
        let opcode = self.memory.get_8_bit_value(location);

        instruction::INSTRUCTION_SET
            .into_iter()
            .find(|i| i.opcode == opcode)
    }

    fn get_address(&self, instruction: Instruction) -> (usize, bool) {
        match instruction.addressing_mode {
            AddressingMode::Accumulator => {
                panic!("Can't get an address for the Accumulator addressing mode.")
            }
            AddressingMode::Implied => {
                panic!("Can't get an address for the Implied addressing mode.")
            }
            AddressingMode::Relative => {
                panic!("Can't get an address for the Relative addressing mode.")
            }
            AddressingMode::Immediate => (self.registers.pc as usize + 1, false),
            AddressingMode::ZeroPage => {
                let zero_page_offset = self
                    .memory
                    .get_8_bit_value((self.registers.pc + 1) as usize);

                (zero_page_offset as usize, false)
            }
            AddressingMode::ZeroPageX => {
                let zero_page_offset = self
                    .memory
                    .get_8_bit_value((self.registers.pc + 1) as usize);

                (zero_page_offset as usize + self.registers.x as usize, false)
            }
            AddressingMode::ZeroPageY => {
                let zero_page_offset = self
                    .memory
                    .get_8_bit_value((self.registers.pc + 1) as usize);

                (zero_page_offset as usize + self.registers.y as usize, false)
            }
            AddressingMode::Absolute => {
                let address = self
                    .memory
                    .get_16_bit_value((self.registers.pc + 1) as usize);

                (address as usize, false)
            }
            AddressingMode::AbsoluteX => {
                let address = self
                    .memory
                    .get_16_bit_value((self.registers.pc + 1) as usize);

                (
                    address as usize + self.registers.x as usize,
                    Cpu::crosses_boundary_by_address_offset(address, self.registers.x),
                )
            }
            AddressingMode::AbsoluteY => {
                let address = self
                    .memory
                    .get_16_bit_value((self.registers.pc + 1) as usize);

                (
                    address as usize + self.registers.y as usize,
                    Cpu::crosses_boundary_by_address_offset(address, self.registers.y),
                )
            }
            AddressingMode::Indirect => {
                let indirect_address = self.memory.get_16_bit_value((self.registers.pc + 1) as usize);
                let address = self.memory.get_16_bit_value(indirect_address as usize);

                (address as usize, false)
            }
            AddressingMode::IndirectX => {
                let indirect_address = self
                    .memory
                    .get_8_bit_value((self.registers.pc + 1) as usize)
                    as usize
                    + self.registers.x as usize;
                let address = self.memory.get_16_bit_value(indirect_address);

                (address as usize, false)
            }
            AddressingMode::IndirectY => {
                let indirect_address = self
                    .memory
                    .get_8_bit_value((self.registers.pc + 1) as usize)
                    as usize;
                let address = self.memory.get_16_bit_value(indirect_address);

                (
                    address as usize + self.registers.y as usize,
                    Cpu::crosses_boundary_by_address_offset(address, self.registers.y),
                )
            }
        }
    }

    fn get_value(&self, instruction: Instruction) -> (u8, bool) {
        let (address, crossed_boundary) = self.get_address(instruction);

        (self.memory.get_8_bit_value(address), crossed_boundary)
    }

    pub fn set_zero_flag(&mut self, value: u8) {
        self.registers.p.zero_flag = value == 0;
    }

    pub fn set_negative_flag(&mut self, value: u8) {
        self.registers.p.negative_flag = value & 0x80 != 0;
    }

    pub fn set_overflow_flag(&mut self, a: u8, b: u8, result: u8) {
        // Overflow occurs if both numbers have the same sign and
        // the result has a different sign.

        // !(a ^ b) - 0x80 bit will be set if both signs are true.
        // (a ^ result) - 0x80 bit will be set if result has a different sign.

        // Based on a StackOverflow answer: https://stackoverflow.com/questions/29193303/6502-emulation-proper-way-to-implement-adc-and-sbc

        self.registers.p.overflow_flag = (!(a ^ b) & (a ^ result) & 0x80) != 0;
    }

    pub fn set_carry_flag(&mut self, result: u16) {
        self.registers.p.carry_flag = result > 0xff;
    }

    pub fn compare(&mut self, register_value: u8, value: u8) {
        let result = (register_value as u16).wrapping_sub(value as u16);
        
        self.set_zero_flag(result as u8);
        self.set_negative_flag(result as u8);
        self.set_carry_flag(!result);
    }

    pub fn crosses_boundary_by_address_offset(address: u16, offset: u8) -> bool {
        address & 0xff00 != (address + offset as u16) &0xff00
    }

    pub fn crosses_boundary_by_two_addresses(base_address: u16, address: u16) -> bool{
        base_address & 0xff00 != address & 0xff00
    }

    pub fn calculate_address_from_relative_offset(base_address: u16, offset: u8) -> u16 {
        match offset & 0x80 {
            0x80 => {
                let positive_offset = !offset + 1;
                base_address - positive_offset as u16
            }
            _ => base_address + offset as u16,
        }
    }

    pub fn pull_u8(&mut self) -> u8 {
        self.registers.sp = self.registers.sp.wrapping_add(1);

        let address = 0x0100 + self.registers.sp as usize;

        self.memory.get_8_bit_value(address)
    }

    pub fn pull_u16(&mut self) -> u16 {
        self.registers.sp = self.registers.sp.wrapping_add(2);

        let address = 0x0100 + self.registers.sp as usize - 1;

        self.memory.get_16_bit_value(address)
    }

    pub fn push_u8(&mut self, value: u8) {
        let stack_pointer: usize = STACK_BASE_ADDRESS + self.registers.sp as usize;

        self.memory.set_8_bit_value(stack_pointer, value);

        self.registers.sp -= 1;
    }

    pub fn push_u16(&mut self, value: u16) {
        let stack_pointer: usize = STACK_BASE_ADDRESS + self.registers.sp as usize;

        self.memory.set_16_bit_value(stack_pointer - 1, value);

        self.registers.sp -= 2;
    }

    pub fn save_register(&mut self, instruction: Instruction, value: u8) -> ExecutionReturnValues {
        let (address, _) = self.get_address(instruction);


        self.memory.set_8_bit_value(address, value);

        ExecutionReturnValues::new(instruction, false)  
    }

    pub fn adc_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let (value, crossed_boundary) = self.get_value(instruction);

        let carry = match self.registers.p.carry_flag {
            true => 1u16,
            false => 0u16,
        };

        let mut result = self.registers.a as u16 + value as u16 + carry;

        if self.registers.p.decimal_flag {
            if (self.registers.a & 0x0f) + (value & 0x0F) + carry as u8 > 9 {
                result += 6;
            }

            if result > 0x99 {
                result += 96;
            }
        }

        self.set_zero_flag(result as u8);
        self.set_negative_flag(result as u8);
        self.set_overflow_flag(self.registers.a, value, result as u8);
        self.set_carry_flag(result);

        self.registers.a = result as u8;

        ExecutionReturnValues::new(instruction, crossed_boundary)
    }

    pub fn and_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let (value, crossed_boundary) = self.get_value(instruction);

        let result = self.registers.a & value;

        self.set_zero_flag(result);
        self.set_negative_flag(result);

        self.registers.a = result;

        ExecutionReturnValues::new(instruction, crossed_boundary)
    }

    pub fn asl_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let (address, value, crossed_boundary): (Option<usize>, u8, bool) =
            match instruction.addressing_mode {
                AddressingMode::Accumulator => (None, self.registers.a, false),
                _ => {
                    let (address, crossed_boundary) = self.get_address(instruction);
                    (
                        Some(address),
                        self.memory.contents[address],
                        crossed_boundary,
                    )
                }
            };

        self.registers.p.carry_flag = value & 0x80 == 0x80;

        let result = value << 1;

        self.set_zero_flag(result);
        self.set_negative_flag(result);

        if address.is_none() {
            self.registers.a = result;
        } else {
            self.memory.contents[address.unwrap()] = result;
        }

        ExecutionReturnValues::new(instruction, crossed_boundary)
    }

    pub fn bcc_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.branch(instruction, !self.registers.p.carry_flag)
    }

    pub fn bcs_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.branch(instruction, self.registers.p.carry_flag)
    }

    pub fn beq_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.branch(instruction, self.registers.p.zero_flag)
    }

    pub fn bit_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let (value, _) = self.get_value(instruction);

        let result = self.registers.a & value;

        self.set_zero_flag(result);
        self.registers.p.negative_flag = value & 0x80 != 0;
        self.registers.p.overflow_flag = value & 0x40 != 0;

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn bmi_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.branch(instruction, self.registers.p.negative_flag)
    }

    pub fn bne_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.branch(instruction, !self.registers.p.zero_flag)
    }

    pub fn bpl_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.branch(instruction, !self.registers.p.negative_flag)
    }
    
    pub fn brk_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.push_u16(self.registers.pc + 2);

        self.registers.p.break_flag = true;
        self.push_u8(self.registers.p.to_byte());
        self.registers.p.break_flag = false;
        
        self.registers.p.interrupt_disable_flag = true;

        self.registers.pc = self.memory.get_16_bit_value(IRQ_BRK_VECTOR);

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn bvc_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.branch(instruction, !self.registers.p.overflow_flag)
    }

    pub fn bvs_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.branch(instruction, self.registers.p.overflow_flag)
    }

    pub fn clc_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.registers.p.carry_flag = false;

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn cld_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.registers.p.decimal_flag = false;

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn cli_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.registers.p.interrupt_disable_flag = false;

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn clv_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.registers.p.overflow_flag = false;

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn cmp_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let (value, crossed_boundary) = self.get_value(instruction);

        self.compare(self.registers.a, value);

        ExecutionReturnValues::new(instruction, crossed_boundary)
    }

    pub fn cpx_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let (value, crossed_boundary) = self.get_value(instruction);

        self.compare(self.registers.x, value);

        ExecutionReturnValues::new(instruction, crossed_boundary)
    }

    pub fn cpy_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let (value, crossed_boundary) = self.get_value(instruction);

        self.compare(self.registers.y, value);

        ExecutionReturnValues::new(instruction, crossed_boundary)
    }

    pub fn dec_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let (address, crossed_boundary) = self.get_address(instruction);

        let result = self.memory.contents[address].wrapping_sub(1);

        self.set_negative_flag(result);
        self.set_zero_flag(result);

        self.memory.contents[address] = result;

        ExecutionReturnValues::new(instruction, crossed_boundary)
    }

    pub fn dex_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let result = self.registers.x.wrapping_sub(1);

        self.set_negative_flag(result);
        self.set_zero_flag(result);

        self.registers.x = result;

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn dey_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let result = self.registers.y.wrapping_sub(1);

        self.set_negative_flag(result);
        self.set_zero_flag(result);

        self.registers.y = result;

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn eor_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let (value, crossed_boundary) = self.get_value(instruction);

        let result = self.registers.a ^ value;
        
        self.set_negative_flag(result);
        self.set_zero_flag(result);

        self.registers.a = result;

        ExecutionReturnValues::new(instruction, crossed_boundary)
    }

    pub fn inc_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let (address, crossed_boundary) = self.get_address(instruction);

        let result = self.memory.contents[address].wrapping_add(1);

        self.set_negative_flag(result);
        self.set_zero_flag(result);

        self.memory.contents[address] = result;

        ExecutionReturnValues::new(instruction, crossed_boundary)
    }

    pub fn inx_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let result = self.registers.x.wrapping_add(1);

        self.set_negative_flag(result);
        self.set_zero_flag(result);

        self.registers.x = result;

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn iny_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let result = self.registers.y.wrapping_add(1);

        self.set_negative_flag(result);
        self.set_zero_flag(result);

        self.registers.y = result;

        ExecutionReturnValues::new(instruction, false)
    }
    
    pub fn jmp_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let (address, _) = self.get_address(instruction);

        self.registers.pc = address as u16;

        ExecutionReturnValues::new(instruction, false)
    }
    
    pub fn jsr_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let (address, _) = self.get_address(instruction);

        self.push_u16(self.registers.pc + 2);

        self.registers.pc = address as u16;

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn lda_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let (value, crossed_boundary) = self.get_value(instruction);

        self.set_zero_flag(value);
        self.set_negative_flag(value);

        self.registers.a = value;

        ExecutionReturnValues::new(instruction, crossed_boundary)
    }

    pub fn ldx_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let (value, crossed_boundary) = self.get_value(instruction);

        self.set_negative_flag(value);
        self.set_zero_flag(value);

        self.registers.x = value;

        ExecutionReturnValues::new(instruction, crossed_boundary)
    }

    pub fn ldy_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let (value, crossed_boundary) = self.get_value(instruction);

        self.set_negative_flag(value);
        self.set_zero_flag(value);

        self.registers.y = value;

        ExecutionReturnValues::new(instruction, crossed_boundary)
    }

    pub fn lsr_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let (address, value, crossed_boundary): (Option<usize>, u8, bool) =
            match instruction.addressing_mode {
                AddressingMode::Accumulator => (None, self.registers.a, false),
                _ => {
                    let (address, crossed_boundary) = self.get_address(instruction);
                    (
                        Some(address),
                        self.memory.contents[address],
                        crossed_boundary,
                    )
                }
            };

        self.registers.p.carry_flag = value & 0x01 == 0x01;

        let result = (value >> 1) & 0x7F;

        self.registers.p.negative_flag = false;
        self.set_zero_flag(result);

        if address.is_none() {
            self.registers.a = result;
        } else {
            self.memory.contents[address.unwrap()] = result;
        }

        ExecutionReturnValues::new(instruction, crossed_boundary)
    }

    pub fn nop_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        ExecutionReturnValues::new(instruction, false)
    }

    pub fn ora_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let (value, crossed_boundary) = self.get_value(instruction);

        let result = self.registers.a | value;
        
        self.set_negative_flag(result);
        self.set_zero_flag(result);

        self.registers.a = result;

        ExecutionReturnValues::new(instruction, crossed_boundary)
    }

    pub fn pha_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.push_u8(self.registers.a);

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn php_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let mut flags = self.registers.p.to_byte();
        flags |= status_flags::UNUSED_FLAG | status_flags::BREAK_FLAG;

        self.push_u8(flags);

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn pla_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.registers.a = self.pull_u8();

        self.set_negative_flag(self.registers.a);
        self.set_zero_flag(self.registers.a);

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn plp_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let mut flags = self.pull_u8();

        flags &= !status_flags::BREAK_FLAG;

        self.registers.p.from_byte(flags);

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn rol_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let (address, value, crossed_boundary): (Option<usize>, u8, bool) =
            match instruction.addressing_mode {
                AddressingMode::Accumulator => (None, self.registers.a, false),
                _ => {
                    let (address, crossed_boundary) = self.get_address(instruction);
                    (
                        Some(address),
                        self.memory.contents[address],
                        crossed_boundary,
                    )
                }
            };

        let high_bit = value & 0x80;

        let mut result = value << 1;

        if self.registers.p.carry_flag {
            result |= 0x01;
        }

        self.registers.p.carry_flag = high_bit == 0x80;

        self.set_negative_flag(result);
        self.set_zero_flag(result);

        if address.is_none() {
            self.registers.a = result;
        } else {
            self.memory.contents[address.unwrap()] = result;
        }

        ExecutionReturnValues::new(instruction, crossed_boundary)
    }

    pub fn ror_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let (address, value, crossed_boundary): (Option<usize>, u8, bool) =
            match instruction.addressing_mode {
                AddressingMode::Accumulator => (None, self.registers.a, false),
                _ => {
                    let (address, crossed_boundary) = self.get_address(instruction);
                    (
                        Some(address),
                        self.memory.contents[address],
                        crossed_boundary,
                    )
                }
            };

        let low_bit = value & 0x01;

        let mut result = value >> 1;

        if self.registers.p.carry_flag {
            result |= 0x80;
        }

        self.registers.p.carry_flag = low_bit == 0x01;

        self.set_negative_flag(result);
        self.set_zero_flag(result);

        if address.is_none() {
            self.registers.a = result;
        } else {
            self.memory.contents[address.unwrap()] = result;
        }

        ExecutionReturnValues::new(instruction, crossed_boundary)
    }

    pub fn rti_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let mut flags = self.pull_u8();

        flags &= !status_flags::BREAK_FLAG;

        self.registers.p.from_byte(flags);

        let address = self.pull_u16();

        self.registers.pc = address;

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn rts_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let address = self.pull_u16();

        self.registers.pc = address + 1;

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn sbc_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        let (value, crossed_boundary) = self.get_value(instruction);

        let carry = match self.registers.p.carry_flag {
            true => 0u16,
            false => 1u16,
        };

        let mut result = (self.registers.a as u16).wrapping_sub(value as u16 + carry);

        if self.registers.p.decimal_flag {
            if (self.registers.a & 0x0f) < (value & 0x0f) + carry as u8 {
                result -= 6;
            }

            if result & 0xFF > 0x99 {
                result -= 96;
            }
        }

        self.set_zero_flag(result as u8);
        self.set_negative_flag(result as u8);
        self.set_overflow_flag(self.registers.a, !value, result as u8);
        self.set_carry_flag(!result);

        self.registers.a = result as u8;

        ExecutionReturnValues::new(instruction, crossed_boundary)
    }

    pub fn sec_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.registers.p.carry_flag = true;

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn sed_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.registers.p.decimal_flag = true;

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn sei_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.registers.p.interrupt_disable_flag = true;

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn sta_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.save_register(instruction, self.registers.a)
    }

    pub fn stx_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.save_register(instruction, self.registers.x)
    }

    pub fn sty_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.save_register(instruction, self.registers.y)
    }

    pub fn tax_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.registers.x = self.registers.a;

        self.set_negative_flag(self.registers.x);
        self.set_zero_flag(self.registers.x);

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn tay_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.registers.y = self.registers.a;

        self.set_negative_flag(self.registers.y);
        self.set_zero_flag(self.registers.y);

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn tsx_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.registers.x = self.registers.sp;

        self.set_negative_flag(self.registers.x);
        self.set_zero_flag(self.registers.x);

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn txa_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.registers.a = self.registers.x;

        self.set_negative_flag(self.registers.a);
        self.set_zero_flag(self.registers.a);

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn txs_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.registers.sp = self.registers.x;

        ExecutionReturnValues::new(instruction, false)
    }

    pub fn tya_instruction(&mut self, instruction: Instruction) -> ExecutionReturnValues {
        self.registers.a = self.registers.y;

        self.set_negative_flag(self.registers.a);
        self.set_zero_flag(self.registers.a);

        ExecutionReturnValues::new(instruction, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_zero_flag_when_not_zero() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.zero_flag = true;

        cpu.set_zero_flag(0xff);

        assert!(!cpu.registers.p.zero_flag);
    }

    #[test]
    fn test_set_zero_flag_when_zero() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.zero_flag = false;

        cpu.set_zero_flag(0x00);

        assert!(cpu.registers.p.zero_flag);
    }

    #[test]
    fn test_set_overflow_flag_when_two_positives_results_in_a_negative() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.overflow_flag = false;

        cpu.set_overflow_flag(0x7f, 0x01, (0x7f + 0x01) as u16 as u8);

        assert!(cpu.registers.p.overflow_flag);
    }

    #[test]
    fn test_set_overflow_flag_when_two_positives_results_in_a_positive() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.overflow_flag = false;

        cpu.set_overflow_flag(0x7e, 0x01, (0x7e + 0x01) as u16 as u8);

        assert!(!cpu.registers.p.overflow_flag);
    }

    #[test]
    fn test_set_overflow_flag_when_two_negatives_results_in_a_positive() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.overflow_flag = false;

        cpu.set_overflow_flag(0x80, 0xff, (0x80 + 0xff) as u16 as u8);

        assert!(cpu.registers.p.overflow_flag);
    }

    #[test]
    fn test_set_overflow_flag_when_two_negatives_results_in_a_negative() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.overflow_flag = false;

        cpu.set_overflow_flag(0x81, 0xff, (0x81 + 0xff) as u16 as u8);

        assert!(!cpu.registers.p.overflow_flag);
    }

    #[test]
    fn test_set_carry_flag_when_no_carry() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.carry_flag = true;

        cpu.set_carry_flag(0x00ff);

        assert!(!cpu.registers.p.carry_flag);
    }

    #[test]
    fn test_set_carry_flag_when_carry() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.carry_flag = false;

        cpu.set_carry_flag(0x0100);

        assert!(cpu.registers.p.carry_flag);
    }

    #[test]
    fn test_crosses_boundary_not_crossed() {
        assert!(!Cpu::crosses_boundary_by_address_offset(0x1ffe, 0x01));
    }

    #[test]
    fn test_crosses_boundary_crossed() {
        assert!(Cpu::crosses_boundary_by_address_offset(0x1fff, 0x01));
    }

    #[test]
    fn test_pull_u8() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.sp = 0xFE;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x01FF] = 0xFF;

        let result = cpu.pull_u8();

        assert_eq!(cpu.registers.sp, 0xFF);
        assert_eq!(result, 0xFF);
    }
    
    #[test]
    fn test_pull_u16() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.sp = 0xFD;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x01FE] = 0xFF;
        cpu.memory.contents[0x01FF] = 0x20;
        
        let result = cpu.pull_u16();

        assert_eq!(cpu.registers.sp, 0xFF);
        assert_eq!(result, 0x20FF);
    }

    #[test]
    fn test_push_u8() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.sp = 0xff;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x01ff] = 0x00;

        cpu.push_u8(0xff);

        assert_eq!(cpu.registers.sp, 0xfe);
        assert_eq!(cpu.memory.contents[0x01ff], 0xff);
    }
    
    #[test]
    fn test_push_u16() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.sp = 0xff;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x01fe] = 0x00;
        cpu.memory.contents[0x01ff] = 0x00;
        cpu.push_u16(0x0102);

        assert_eq!(cpu.registers.sp, 0xfd);
        assert_eq!(cpu.memory.contents[0x01fe], 0x02);
        assert_eq!(cpu.memory.contents[0x01ff], 0x01);
    }

    #[test]
    fn test_compare_when_register_is_less_than_value() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x10;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.carry_flag = true;

        cpu.compare(cpu.registers.a, 0x11);


        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.carry_flag);
    }

    #[test]
    fn test_compare_when_register_is_equal_to_value() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x10;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.zero_flag = false;
        cpu.registers.p.carry_flag = false;

        cpu.compare(cpu.registers.a, 0x10);


        assert!(!cpu.registers.p.negative_flag);
        assert!(cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.carry_flag);
    }

    #[test]
    fn test_compare_when_register_is_greater_than_value() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x10;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.carry_flag = false;

        cpu.compare(cpu.registers.a, 0x09);


        assert!(!cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.carry_flag);
    }

    #[test]
    fn test_00_brk_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.sp = 0xff;
        cpu.registers.pc = 0x8000;

        let mut old_flags = cpu.registers.p.clone();
        old_flags.break_flag = true;
        let old_flags = old_flags.to_byte();
        
        cpu.memory.contents[0x8000] = 0;
        cpu.memory.contents[IRQ_BRK_VECTOR] = 0x02;
        cpu.memory.contents[IRQ_BRK_VECTOR + 1] = 0x40;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x4002);
        assert_eq!(cpu.registers.sp, 0xfc);
        assert_eq!(cpu.memory.contents[0x01fd], old_flags);
        assert_eq!(cpu.memory.contents[0x01fe], 0x02);
        assert_eq!(cpu.memory.contents[0x01ff], 0x80);
    }

    #[test]
    fn test_01_ora_indirect_x_instruction_carry() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x22;
        cpu.registers.x = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0x00;
        cpu.memory.contents[0x0033] = 0x30;
        cpu.memory.contents[0x3000] = 0x55;
        cpu.memory.contents[0x8000] = 0x01;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x77);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 6);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_05_ora_zero_page_instruction_carry() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x22;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0x55;
        cpu.memory.contents[0x8000] = 0x05;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x77);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 3);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_06_asl_zero_page_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0xcc;
        cpu.memory.contents[0x8000] = 0x06;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x0030], 0x98);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert!(cpu.registers.p.carry_flag);
        assert!(!return_values.set_program_counter);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 5);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_08_php_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.sp = 0xFF;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x01FF] = 0x00;
        cpu.memory.contents[0x8000] = 0x08;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x01FF], cpu.registers.p.to_byte() | status_flags::UNUSED_FLAG | status_flags::BREAK_FLAG);
        assert_eq!(cpu.registers.sp, 0xFE);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 3);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_09_ora_immediate_instruction_carry() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x22;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x09;
        cpu.memory.contents[0x8001] = 0x55;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x77);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_0a_asl_accumulator_instruction_carry() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xcc;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x0a;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x98);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert!(cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_0a_asl_accumulator_instruction_no_carry() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x4c;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x0a;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x98);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_0d_ora_absolute_instruction_carry() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x22;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3000] = 0x55;
        cpu.memory.contents[0x8000] = 0x0D;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x77);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }


    #[test]
    fn test_0e_asl_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x4000] = 0xcc;
        cpu.memory.contents[0x8000] = 0x0e;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x40;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x4000], 0x98);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert!(cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 6);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_10_bpl_relative_instruction_with_negative_set() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x10;
        cpu.memory.contents[0x8001] = 0x02;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x8002);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_10_bpl_relative_instruction_with_negative_not_set() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x10;
        cpu.memory.contents[0x8001] = 0x02;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x8004);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_11_ora_indirect_y_instruction_carry() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x22;
        cpu.registers.y = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0x00;
        cpu.memory.contents[0x0031] = 0x30;
        cpu.memory.contents[0x3002] = 0x55;
        cpu.memory.contents[0x8000] = 0x11;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x77);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 5);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_15_ora_zero_page_x_instruction_carry() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x22;
        cpu.registers.x = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0x55;
        cpu.memory.contents[0x8000] = 0x15;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x77);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_16_asl_zero_page_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0xcc;
        cpu.memory.contents[0x8000] = 0x16;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x0032], 0x98);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert!(cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 6);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_18_clc_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x18;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_19_ora_absolute_y_instruction_carry() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x22;
        cpu.registers.y = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3002] = 0x55;
        cpu.memory.contents[0x8000] = 0x19;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x77);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_1d_ora_absolute_x_instruction_carry() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x22;
        cpu.registers.x = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3002] = 0x55;
        cpu.memory.contents[0x8000] = 0x1D;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x77);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_1e_asl_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 2;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x4002] = 0xcc;
        cpu.memory.contents[0x8000] = 0x1e;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x40;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x4002], 0x98);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert!(cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 7);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_20_jsr_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.sp = 0xFF;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x20;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x3000);
        assert_eq!(cpu.registers.sp, 0xFD);
        assert_eq!(cpu.memory.contents[0x01FE], 0x02);
        assert_eq!(cpu.memory.contents[0x01FF], 0x80);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 6);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_21_and_indirect_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xef;
        cpu.registers.x = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0x00;
        cpu.memory.contents[0x0033] = 0x40;

        cpu.memory.contents[0x4000] = 0xfe;
        cpu.memory.contents[0x8000] = 0x21;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xee);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 6);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_24_bit_zero_page_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xff;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0xCF;
        cpu.memory.contents[0x8000] = 0x24;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xff);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert!(cpu.registers.p.overflow_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 3);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_25_and_zero_page_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xef;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0xfe;
        cpu.memory.contents[0x8000] = 0x25;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xee);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 3);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_26_rol_zero_page_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.carry_flag = true;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0xCF;
        cpu.memory.contents[0x8000] = 0x26;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x0030], 0x9F);
        assert!(cpu.registers.p.carry_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 5);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_28_plp_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.break_flag = true;
        cpu.registers.sp = 0xFE;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x01FF] = status_flags::UNUSED_FLAG | status_flags::BREAK_FLAG;
        cpu.memory.contents[0x8000] = 0x28;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.sp, 0xFF);
        assert!(!cpu.registers.p.break_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_29_and_immediate_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xef;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x29;
        cpu.memory.contents[0x8001] = 0xfe;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xee);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_2a_rol_accumulator_instruction_without_carry() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x4F;
        cpu.registers.p.carry_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x2A;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x9E);
        assert!(!cpu.registers.p.carry_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_2a_rol_accumulator_instruction_with_carry() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xCF;
        cpu.registers.p.carry_flag = true;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x2A;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x9F);
        assert!(cpu.registers.p.carry_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_2c_bit_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xff;
        cpu.registers.p.zero_flag = false;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.overflow_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x4000] = 0x00;
        cpu.memory.contents[0x8000] = 0x2c;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x40;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xff);
        assert!(cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.overflow_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_2d_and_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xef;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3000] = 0xfe;
        cpu.memory.contents[0x8000] = 0x2d;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xee);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_2e_rol_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.carry_flag = true;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3000] = 0xCF;
        cpu.memory.contents[0x8000] = 0x2E;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x3000], 0x9F);
        assert!(cpu.registers.p.carry_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 6);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_30_bmi_relative_instruction_with_negative_not_set() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x30;
        cpu.memory.contents[0x8001] = 0x02;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x8002);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_30_bmi_relative_instruction_with_negative_set() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x30;
        cpu.memory.contents[0x8001] = 0x02;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x8004);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_31_and_indirect_y_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xef;
        cpu.registers.y = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0x00;
        cpu.memory.contents[0x0031] = 0x40;

        cpu.memory.contents[0x4002] = 0xfe;
        cpu.memory.contents[0x8000] = 0x31;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xee);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 5);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_35_and_zero_page_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xef;
        cpu.registers.x = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0xfe;
        cpu.memory.contents[0x8000] = 0x35;
        cpu.memory.contents[0x8001] = 0x30;


        #[test]
        fn test_2a_rol_accumulator_instruction_without_carry() {
            let mut cpu: Cpu = Cpu::new(0x8000);
            cpu.registers.a = 0x4F;
            cpu.registers.p.carry_flag = false;
            cpu.registers.p.zero_flag = true;
            cpu.registers.p.negative_flag = false;
            cpu.registers.pc = 0x8000;
    
            cpu.memory.contents[0x8000] = 0x2A;
    
            let option_return_values = cpu.execute_opcode();
    
            assert!(option_return_values.is_some());
    
            let return_values = option_return_values.unwrap();
    
            assert_eq!(cpu.registers.a, 0x9E);
            assert!(!cpu.registers.p.carry_flag);
            assert!(!cpu.registers.p.zero_flag);
            assert!(cpu.registers.p.negative_flag);
            assert_eq!(return_values.bytes, 1);
            assert_eq!(return_values.clock_periods, 2);
            assert!(!return_values.set_program_counter);
        }
    
        #[test]
        fn test_2a_rol_accumulator_instruction_with_carry() {
            let mut cpu: Cpu = Cpu::new(0x8000);
            cpu.registers.a = 0xCF;
            cpu.registers.p.carry_flag = true;
            cpu.registers.p.zero_flag = true;
            cpu.registers.p.negative_flag = false;
            cpu.registers.pc = 0x8000;
    
            cpu.memory.contents[0x8000] = 0x2A;
    
            let option_return_values = cpu.execute_opcode();
    
            assert!(option_return_values.is_some());
    
            let return_values = option_return_values.unwrap();
    
            assert_eq!(cpu.registers.a, 0x9F);
            assert!(cpu.registers.p.carry_flag);
            assert!(!cpu.registers.p.zero_flag);
            assert!(cpu.registers.p.negative_flag);
            assert_eq!(return_values.bytes, 1);
            assert_eq!(return_values.clock_periods, 2);
            assert!(!return_values.set_program_counter);
        }
        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xee);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_36_rol_zero_page_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x02;
        cpu.registers.p.carry_flag = true;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0xCF;
        cpu.memory.contents[0x8000] = 0x36;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x0032], 0x9F);
        assert!(cpu.registers.p.carry_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 6);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_38_sec_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x38;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_39_and_absolute_y_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xef;
        cpu.registers.y = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3002] = 0xfe;
        cpu.memory.contents[0x8000] = 0x39;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xee);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_3d_and_absolute_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xef;
        cpu.registers.x = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3002] = 0xfe;
        cpu.memory.contents[0x8000] = 0x3d;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xee);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_3e_rol_absolute_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x02;
        cpu.registers.p.carry_flag = true;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3002] = 0xCF;
        cpu.memory.contents[0x8000] = 0x3E;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x3002], 0x9F);
        assert!(cpu.registers.p.carry_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 7);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_40_rti_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.break_flag = true;
        cpu.registers.sp = 0xFC;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x01FD] = status_flags::UNUSED_FLAG | status_flags::BREAK_FLAG;
        cpu.memory.contents[0x01FE] = 0x03;
        cpu.memory.contents[0x01FF] = 0x30;
        cpu.memory.contents[0x8000] = 0x40;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.sp, 0xFF);
        assert_eq!(cpu.registers.pc, 0x3003);
        assert!(!cpu.registers.p.break_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 6);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_41_eor_indirect_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xCC;
        cpu.registers.x = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0x00;
        cpu.memory.contents[0x0033] = 0x40;
        cpu.memory.contents[0x4000] = 0xEE;
        cpu.memory.contents[0x8000] = 0x41;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x22);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 6);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_45_eor_zero_page_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xCC;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0xEE;
        cpu.memory.contents[0x8000] = 0x45;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x22);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 3);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_46_lsr_zero_page_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.carry_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0xCF;
        cpu.memory.contents[0x8000] = 0x46;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x0030], 0x67);
        assert!(cpu.registers.p.carry_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 5);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_48_pha_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xFF;
        cpu.registers.sp = 0xFF;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x01FF] = 0x00;
        cpu.memory.contents[0x8000] = 0x48;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x01FF], 0xFF);
        assert_eq!(cpu.registers.sp, 0xFE);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 3);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_49_eor_immediate_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xCC;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x49;
        cpu.memory.contents[0x8001] = 0xEE;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x22);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_4a_lsr_accumulator_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xCF;
        cpu.registers.p.carry_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x4A;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x67);
        assert!(cpu.registers.p.carry_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_4c_jmp_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x4C;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x3000);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 3);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_4d_eor_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xCC;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3000] = 0xEE;
        cpu.memory.contents[0x8000] = 0x4D;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x22);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_4e_lsr_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.carry_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3000] = 0xCF;
        cpu.memory.contents[0x8000] = 0x4e;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x3000], 0x67);
        assert!(cpu.registers.p.carry_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 6);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_50_bvc_relative_instruction_with_overflow_not_set() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.overflow_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x50;
        cpu.memory.contents[0x8001] = 0x02;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x8004);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_50_bvc_relative_instruction_with_overflow_set() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.overflow_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x50;
        cpu.memory.contents[0x8001] = 0x02;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x8002);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_51_eor_indirect_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xCC;
        cpu.registers.y = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0x00;
        cpu.memory.contents[0x0031] = 0x40;
        cpu.memory.contents[0x4002] = 0xEE;
        cpu.memory.contents[0x8000] = 0x51;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x22);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 5);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_55_eor_zero_page_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xCC;
        cpu.registers.x = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0xEE;
        cpu.memory.contents[0x8000] = 0x55;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x22);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_56_lsr_zero_page_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x02;
        cpu.registers.p.carry_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0xCF;
        cpu.memory.contents[0x8000] = 0x56;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x0032], 0x67);
        assert!(cpu.registers.p.carry_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 6);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_58_cli_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.interrupt_disable_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x58;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_59_eor_absolute_y_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xCC;
        cpu.registers.y = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3002] = 0xEE;
        cpu.memory.contents[0x8000] = 0x59;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x22);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_5d_eor_absolute_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xCC;
        cpu.registers.x = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3002] = 0xEE;
        cpu.memory.contents[0x8000] = 0x5D;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x22);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_5e_lsr_absolute_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x02;
        cpu.registers.p.carry_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3002] = 0xCF;
        cpu.memory.contents[0x8000] = 0x5e;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x3002], 0x67);
        assert!(cpu.registers.p.carry_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 7);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_60_rts_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.sp = 0xFD;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x01FE] = 0x03;
        cpu.memory.contents[0x01FF] = 0x30;
        cpu.memory.contents[0x8000] = 0x60;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x3004);
        assert!(!cpu.registers.p.break_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 6);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_61_adc_indirect_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x40;
        cpu.registers.x = 0x02;
        cpu.registers.p.zero_flag = false;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0x00;
        cpu.memory.contents[0x0033] = 0x40;
        cpu.memory.contents[0x4000] = 0x20;
        cpu.memory.contents[0x8000] = 0x61;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x60);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.overflow_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 6);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_65_adc_zero_page_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x40;
        cpu.registers.p.zero_flag = false;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0x20;
        cpu.memory.contents[0x8000] = 0x65;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x60);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.overflow_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 3);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_66_ror_zero_page_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.carry_flag = true;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0xCF;
        cpu.memory.contents[0x8000] = 0x66;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x0030], 0xE7);
        assert!(cpu.registers.p.carry_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 5);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_68_pla_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x00;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.sp = 0xFE;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x01FF] = 0xFF;
        cpu.memory.contents[0x8000] = 0x68;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xFF);
        assert_eq!(cpu.registers.sp, 0xFF);
        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_69_adc_immediate_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x99;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x69;
        cpu.memory.contents[0x8001] = 0x99;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x32);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert!(cpu.registers.p.overflow_flag);
        assert!(cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_69_adc_immediate_instruction_with_carry_set_adds_correctly() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xff;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x69;
        cpu.memory.contents[0x8001] = 0x01;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x01);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.overflow_flag);
        assert!(cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_69_adc_immediate_instruction_should_set_carry() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xff;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x69;
        cpu.memory.contents[0x8001] = 0x01;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x00);
        assert!(cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.overflow_flag);
        assert!(cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_69_adc_immediate_instruction_should_not_set_carry() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xfe;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x69;
        cpu.memory.contents[0x8001] = 0x01;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xff);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.overflow_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_69_adc_immediate_instruction_should_overflow_1() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x7f; // 127d
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x69;
        cpu.memory.contents[0x8001] = 0x01; // 127d + 1d = 128d, which is an overflow.

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x80);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert!(cpu.registers.p.overflow_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_69_adc_immediate_instruction_should_not_overflow_1() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x7e; // 126 decimal
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x69;
        cpu.memory.contents[0x8001] = 0x01; // 126d + 1d = 127d, which is not an overflow.

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x7f);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.overflow_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_69_adc_immediate_instruction_should_overflow_2() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x81; // -127
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x69;
        cpu.memory.contents[0x8001] = 0xfe; // -127d + -2d = -129d, which is an overflow.

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x7f);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert!(cpu.registers.p.overflow_flag);
        assert!(cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_69_adc_immediate_instruction_should_not_overflow_2() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x81; // -127 decimal
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x69;
        cpu.memory.contents[0x8001] = 0xff; // -127d + -1d = -128d, which is not an overflow.

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x80);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.overflow_flag);
        assert!(cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_6a_ror_accumulator_instruction_without_carry() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xCE;
        cpu.registers.p.carry_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x6A;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x67);
        assert!(!cpu.registers.p.carry_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_6a_ror_accumulator_instruction_with_carry() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xCF;
        cpu.registers.p.carry_flag = true;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x6A;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xE7);
        assert!(cpu.registers.p.carry_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_6c_jmp_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3000] = 0x00;
        cpu.memory.contents[0x3001] = 0x40;
        cpu.memory.contents[0x8000] = 0x6C;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x4000);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 5);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_6d_adc_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x40;
        cpu.registers.p.zero_flag = false;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3000] = 0x20;
        cpu.memory.contents[0x8000] = 0x6d;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x60);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.overflow_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_6e_ror_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.carry_flag = true;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3000] = 0xCF;
        cpu.memory.contents[0x8000] = 0x6E;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x3000], 0xE7);
        assert!(cpu.registers.p.carry_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 6);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_70_bvs_relative_instruction_with_overflow_not_set() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.overflow_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x70;
        cpu.memory.contents[0x8001] = 0x02;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x8002);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_70_bvs_relative_instruction_with_overflow_set() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.overflow_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x70;
        cpu.memory.contents[0x8001] = 0x02;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x8004);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_71_adc_indirect_y_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x40;
        cpu.registers.y = 0x02;
        cpu.registers.p.zero_flag = false;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0x00;
        cpu.memory.contents[0x0031] = 0x40;
        cpu.memory.contents[0x4002] = 0x20;
        cpu.memory.contents[0x8000] = 0x71;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x60);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.overflow_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 5);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_75_adc_zero_page_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x40;
        cpu.registers.x = 0x03;
        cpu.registers.p.zero_flag = false;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0033] = 0x20;
        cpu.memory.contents[0x8000] = 0x75;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x60);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.overflow_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_76_ror_zero_page_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x02;
        cpu.registers.p.carry_flag = true;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0xCF;
        cpu.memory.contents[0x8000] = 0x76;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x0032], 0xE7);
        assert!(cpu.registers.p.carry_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 6);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_78_sei_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.interrupt_disable_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x78;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(cpu.registers.p.interrupt_disable_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_79_adc_absolute_y_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x40;
        cpu.registers.y = 0x03;
        cpu.registers.p.zero_flag = false;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3003] = 0x20;
        cpu.memory.contents[0x8000] = 0x79;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x60);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.overflow_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_7d_adc_absolute_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x40;
        cpu.registers.x = 0x03;
        cpu.registers.p.zero_flag = false;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3003] = 0x20;
        cpu.memory.contents[0x8000] = 0x7d;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x60);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.overflow_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_7e_ror_absolute_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x02;
        cpu.registers.p.carry_flag = true;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3002] = 0xCF;
        cpu.memory.contents[0x8000] = 0x7E;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x3002], 0xE7);
        assert!(cpu.registers.p.carry_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 7);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_81_sta_indirect_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xFF;
        cpu.registers.x = 0x02;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0x00;
        cpu.memory.contents[0x0033] = 0x40;
        cpu.memory.contents[0x4000] = 0x00;
        cpu.memory.contents[0x8000] = 0x81;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x4000], 0xFF);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 6);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_84_sty_zero_page_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.y = 0xFF;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0x00;
        cpu.memory.contents[0x8000] = 0x84;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x0030], 0xFF);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 3);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_85_sta_zero_page_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xFF;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0x00;
        cpu.memory.contents[0x8000] = 0x85;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x0030], 0xFF);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 3);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_86_stx_zero_page_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0xFF;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0x00;
        cpu.memory.contents[0x8000] = 0x86;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x0030], 0xFF);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 3);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_88_dey_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.y = 0x00;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x88;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.y, 0xFF);
        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_8a_txa_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);

        cpu.registers.x = 0xFF;
        cpu.registers.a = 0x00;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x8A;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xFF);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_8c_sty_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.y = 0xFF;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3000] = 0x00;
        cpu.memory.contents[0x8000] = 0x8C;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x3000], 0xFF);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_8d_sta_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xFF;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3000] = 0x00;
        cpu.memory.contents[0x8000] = 0x8D;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x3000], 0xFF);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_8e_stx_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0xFF;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3000] = 0x00;
        cpu.memory.contents[0x8000] = 0x8E;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x3000], 0xFF);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_90_bcc_relative_instruction_with_carry_not_set() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x90;
        cpu.memory.contents[0x8001] = 0x02;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x8004);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_90_bcc_relative_instruction_with_carry_set() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x90;
        cpu.memory.contents[0x8001] = 0x02;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x8002);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_91_sta_indirect_y_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xFF;
        cpu.registers.y = 0x02;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0x00;
        cpu.memory.contents[0x0031] = 0x40;
        cpu.memory.contents[0x4002] = 0x00;
        cpu.memory.contents[0x8000] = 0x91;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x4002], 0xFF);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 6);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_94_sty_zero_page_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.y = 0xFF;
        cpu.registers.x = 0x02;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0x00;
        cpu.memory.contents[0x8000] = 0x94;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x0032], 0xFF);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_95_sta_zero_page_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xFF;
        cpu.registers.x = 0x02;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0x00;
        cpu.memory.contents[0x8000] = 0x95;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x0032], 0xFF);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_96_stx_zero_page_y_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0xFF;
        cpu.registers.y = 0x02;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0x00;
        cpu.memory.contents[0x8000] = 0x96;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x0032], 0xFF);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_98_tya_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);

        cpu.registers.y = 0xFF;
        cpu.registers.a = 0x00;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x98;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xFF);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_99_sta_absolute_y_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xFF;
        cpu.registers.y = 0x02;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3002] = 0x00;
        cpu.memory.contents[0x8000] = 0x99;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x3002], 0xFF);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 5);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_9a_txs_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0xFF;
        cpu.registers.sp = 0x00;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0x9A;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.sp, 0xFF);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }


    #[test]
    fn test_9d_sta_absolute_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0xFF;
        cpu.registers.x = 0x02;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3002] = 0x00;
        cpu.memory.contents[0x8000] = 0x9D;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x3002], 0xFF);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 5);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_a0_ldy_immediate_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.y = 0x00;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xA0;
        cpu.memory.contents[0x8001] = 0xFF;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.y, 0xFF);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_a1_lda_indirect_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x00;
        cpu.registers.x = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0x00;
        cpu.memory.contents[0x0033] = 0x40;
        cpu.memory.contents[0x4000] = 0xFF;
        cpu.memory.contents[0x8000] = 0xA1;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xFF);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 6);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_a2_ldx_immediate_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x00;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xA2;
        cpu.memory.contents[0x8001] = 0xFF;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.x, 0xff);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_a4_ldy_zero_page_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.y = 0x00;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0xFF;
        cpu.memory.contents[0x8000] = 0xA4;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.y, 0xFF);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 3);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_a5_lda_zero_page_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);

        cpu.registers.a = 0x00;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x50] = 0xFF;
        cpu.memory.contents[0x8000] = 0xA5;
        cpu.memory.contents[0x8001] = 0x50;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xFF);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 3);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_a6_ldx_zero_page_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x00;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0xFF;
        cpu.memory.contents[0x8000] = 0xA6;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.x, 0xff);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 3);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_a8_tay_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);

        cpu.registers.a = 0xFF;
        cpu.registers.y = 0x00;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xA8;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.y, 0xFF);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_a9_lda_immediate_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);

        cpu.registers.a = 0x00;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xa9;
        cpu.memory.contents[0x8001] = 0xff;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xff);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_aa_tax_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);

        cpu.registers.a = 0xFF;
        cpu.registers.x = 0x00;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xAA;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.x, 0xFF);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_ac_ldy_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.y = 0x00;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3000] = 0xFF;
        cpu.memory.contents[0x8000] = 0xAC;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.y, 0xFF);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_ad_lda_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);

        cpu.registers.a = 0x00;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3000] = 0xff;
        cpu.memory.contents[0x8000] = 0xad;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xff);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_ae_ldx_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x00;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3000] = 0xFF;
        cpu.memory.contents[0x8000] = 0xAE;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.x, 0xFF);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_b0_bcs_relative_instruction_with_carry_not_set() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xB0;
        cpu.memory.contents[0x8001] = 0x02;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x8002);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_b0_bcs_relative_instruction_with_carry_set() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xB0;
        cpu.memory.contents[0x8001] = 0x02;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x8004);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_b1_lda_indirect_y_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x00;
        cpu.registers.y = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0x00;
        cpu.memory.contents[0x0031] = 0x40;
        cpu.memory.contents[0x4002] = 0xFF;
        cpu.memory.contents[0x8000] = 0xB1;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xFF);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 5);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_b4_ldy_zero_page_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.y = 0x00;
        cpu.registers.x = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0xFF;
        cpu.memory.contents[0x8000] = 0xB4;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.y, 0xFF);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_b5_lda_zero_page_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);

        cpu.registers.a = 0xff;
        cpu.registers.x = 0x02;
        cpu.registers.p.zero_flag = false;
        cpu.registers.p.negative_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x32] = 0x00;
        cpu.memory.contents[0x8000] = 0xb5;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x00);
        assert!(cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_b6_ldx_zero_page_y_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x00;
        cpu.registers.y = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0xFF;
        cpu.memory.contents[0x8000] = 0xB6;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.x, 0xFF);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_b8_clv_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.overflow_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xb8;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_b9_lda_absolute_y_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);

        cpu.registers.a = 0x00;
        cpu.registers.y = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3002] = 0xff;
        cpu.memory.contents[0x8000] = 0xb9;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xff);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_ba_tsx_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.sp = 0xFF;
        cpu.registers.x = 0x00;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xBA;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.x, 0xFF);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_bc_ldy_absolut_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.y = 0x00;
        cpu.registers.x = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3002] = 0xFF;
        cpu.memory.contents[0x8000] = 0xBC;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.y, 0xFF);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_bd_lda_absolute_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);

        cpu.registers.a = 0x00;
        cpu.registers.x = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3002] = 0xff;
        cpu.memory.contents[0x8000] = 0xbd;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xff);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_be_ldx_absolut_y_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x00;
        cpu.registers.y = 0x02;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3002] = 0xFF;
        cpu.memory.contents[0x8000] = 0xBE;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.x, 0xFF);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_c0_cpy_immediate_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.y = 0x10;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xC0;
        cpu.memory.contents[0x8001] = 0x11;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }


    #[test]
    fn test_c1_cmp_indirect_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x10;
        cpu.registers.x = 0x02;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0x00;
        cpu.memory.contents[0x0033] = 0x30;
        cpu.memory.contents[0x3000] = 0x11;
        cpu.memory.contents[0x8000] = 0xc1;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 6);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_c4_cpy_zero_page_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.y = 0x10;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0x11;
        cpu.memory.contents[0x8000] = 0xC4;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 3);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_c5_cmp_zero_page_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x10;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0x11;
        cpu.memory.contents[0x8000] = 0xc5;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 3);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_c6_dec_zero_page_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0x00;
        cpu.memory.contents[0x8000] = 0xC6;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x0030], 0xFF);
        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 5);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_c8_iny_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.y = 0xFF;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xC8;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.y, 0x00);
        assert!(!cpu.registers.p.negative_flag);
        assert!(cpu.registers.p.zero_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_c9_cmp_immediate_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x10;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xc9;
        cpu.memory.contents[0x8001] = 0x11;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_ca_dex_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x00;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xCA;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.x, 0xFF);
        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_cc_cpy_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.y = 0x10;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3000] = 0x11;
        cpu.memory.contents[0x8000] = 0xCC;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_cd_cmp_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x10;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3000] = 0x11;
        cpu.memory.contents[0x8000] = 0xcd;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_ce_dec_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3000] = 0x00;
        cpu.memory.contents[0x8000] = 0xce;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x3000], 0xFF);
        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 6);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_d0_bne_relative_instruction_with_zero_set() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.zero_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xd0;
        cpu.memory.contents[0x8001] = 0x02;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x8002);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_d0_bne_relative_instruction_with_zero_not_set() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.zero_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xd0;
        cpu.memory.contents[0x8001] = 0x02;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x8004);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_d1_cmp_indirect_y_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x10;
        cpu.registers.y = 0x02;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0x00;
        cpu.memory.contents[0x0031] = 0x30;
        cpu.memory.contents[0x3002] = 0x11;
        cpu.memory.contents[0x8000] = 0xd1;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 5);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_d5_cmp_zero_page_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x10;
        cpu.registers.x = 0x02;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0x11;
        cpu.memory.contents[0x8000] = 0xd5;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_d6_dec_zero_page_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x02;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0x00;
        cpu.memory.contents[0x8000] = 0xD6;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x0032], 0xFF);
        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 6);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_d8_cld_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.decimal_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xd8;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(!cpu.registers.p.decimal_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_d9_cmp_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x10;
        cpu.registers.y = 0x02;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3002] = 0x11;
        cpu.memory.contents[0x8000] = 0xD9;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_dd_cmp_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x10;
        cpu.registers.x = 0x02;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3002] = 0x11;
        cpu.memory.contents[0x8000] = 0xDD;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_de_dec_absolute_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x02;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3002] = 0x00;
        cpu.memory.contents[0x8000] = 0xde;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x3002], 0xFF);
        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 7);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_e0_cpx_immediate_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x10;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xE0;
        cpu.memory.contents[0x8001] = 0x11;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_e4_cpx_zero_page_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x10;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0x11;
        cpu.memory.contents[0x8000] = 0xE4;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 3);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_e6_inc_zero_page_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0030] = 0xFF;
        cpu.memory.contents[0x8000] = 0xE6;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x0030], 0x00);
        assert!(!cpu.registers.p.negative_flag);
        assert!(cpu.registers.p.zero_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 5);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_e8_inx_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0xFF;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xE8;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.x, 0x00);
        assert!(!cpu.registers.p.negative_flag);
        assert!(cpu.registers.p.zero_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_e9_sbc_immediate_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x80;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xe9;
        cpu.memory.contents[0x8001] = 0x02;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x7e);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert!(cpu.registers.p.overflow_flag);
        assert!(cpu.registers.p.carry_flag); // no borrow
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_e9_sbc_immediate_instruction_with_borrow() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x80;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xe9;
        cpu.memory.contents[0x8001] = 0x02;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x7d);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert!(cpu.registers.p.overflow_flag);
        assert!(cpu.registers.p.carry_flag); // no borrow
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_e9_sbc_immediate_instruction_sets_borrow() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x04;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xe9;
        cpu.memory.contents[0x8001] = 0x0a;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0xfa);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.overflow_flag);
        assert!(!cpu.registers.p.carry_flag); // no borrow
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_e9_sbc_immediate_instruction_should_overflow_1() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x80; // -128d
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xe9;
        cpu.memory.contents[0x8001] = 0x01; // -128d - 1d = -129d, which is an overflow

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x7f);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert!(cpu.registers.p.overflow_flag);
        assert!(cpu.registers.p.carry_flag); // no borrow
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_e9_sbc_immediate_instruction_should_not_overflow_1() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x80; // -128d
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xe9;
        cpu.memory.contents[0x8001] = 0x00; // -128d - 0d = -128d, which is not an overflow

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x80);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.overflow_flag);
        assert!(cpu.registers.p.carry_flag); // no borrow
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_e9_sbc_immediate_instruction_should_overflow_2() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x7f; // 128d
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xe9;
        cpu.memory.contents[0x8001] = 0xff; // 127d - -1d = 128d, which is an overflow

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x80);
        assert!(!cpu.registers.p.zero_flag);
        assert!(cpu.registers.p.negative_flag);
        assert!(cpu.registers.p.overflow_flag);
        assert!(!cpu.registers.p.carry_flag); // borrow
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_e9_sbc_immediate_instruction_should_not_overflow_2() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.a = 0x7f; // 127d
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.negative_flag = true;
        cpu.registers.p.overflow_flag = false;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xe9;
        cpu.memory.contents[0x8001] = 0x00; // 127d - 0d = 127d, which is not an overflow

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.a, 0x7f);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.overflow_flag);
        assert!(cpu.registers.p.carry_flag); // no borrow
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_ea_nop_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x10;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xEA;
        cpu.memory.contents[0x8001] = 0x00;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_ec_cpx_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x10;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.p.carry_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3000] = 0x11;
        cpu.memory.contents[0x8000] = 0xEC;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(cpu.registers.p.negative_flag);
        assert!(!cpu.registers.p.zero_flag);
        assert!(!cpu.registers.p.carry_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 4);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_ee_inc_absolute_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3000] = 0xFF;
        cpu.memory.contents[0x8000] = 0xEE;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x3000], 0x00);
        assert!(!cpu.registers.p.negative_flag);
        assert!(cpu.registers.p.zero_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 6);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_f0_beq_relative_instruction_with_zero_not_set() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.zero_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xf0;
        cpu.memory.contents[0x8001] = 0x02;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x8002);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_f0_beq_relative_instruction_with_zero_set() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.zero_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xF0;
        cpu.memory.contents[0x8001] = 0x02;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.registers.pc, 0x8004);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 2);
        assert!(return_values.set_program_counter);
    }

    #[test]
    fn test_f6_inc_zero_page_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x02;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x0032] = 0xFF;
        cpu.memory.contents[0x8000] = 0xF6;
        cpu.memory.contents[0x8001] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x0032], 0x00);
        assert!(!cpu.registers.p.negative_flag);
        assert!(cpu.registers.p.zero_flag);
        assert_eq!(return_values.bytes, 2);
        assert_eq!(return_values.clock_periods, 6);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_f8_sed_implied_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.p.decimal_flag = false;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x8000] = 0xF8;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert!(cpu.registers.p.decimal_flag);
        assert_eq!(return_values.bytes, 1);
        assert_eq!(return_values.clock_periods, 2);
        assert!(!return_values.set_program_counter);
    }

    #[test]
    fn test_fe_inc_absolute_x_instruction() {
        let mut cpu: Cpu = Cpu::new(0x8000);
        cpu.registers.x = 0x02;
        cpu.registers.p.negative_flag = false;
        cpu.registers.p.zero_flag = true;
        cpu.registers.pc = 0x8000;

        cpu.memory.contents[0x3002] = 0xFF;
        cpu.memory.contents[0x8000] = 0xFE;
        cpu.memory.contents[0x8001] = 0x00;
        cpu.memory.contents[0x8002] = 0x30;

        let option_return_values = cpu.execute_opcode();

        assert!(option_return_values.is_some());

        let return_values = option_return_values.unwrap();

        assert_eq!(cpu.memory.contents[0x3002], 0x00);
        assert!(!cpu.registers.p.negative_flag);
        assert!(cpu.registers.p.zero_flag);
        assert_eq!(return_values.bytes, 3);
        assert_eq!(return_values.clock_periods, 7);
        assert!(!return_values.set_program_counter);
    }

}