use super::Cpu;

#[derive(Copy, Clone)]
pub enum AddressingMode {
    Accumulator,
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
            clock_periods: match crossed_boundary {
                true => instruction.clock_periods + 1,
                false => instruction.clock_periods,
            },
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

const ADC_INSTRUCTION: &str = "ADC";
const AND_INSTRUCTION: &str = "AND";
const ASL_INSTRUCTION: &str = "ASL";
const LDA_INSTRUCTION: &str = "LDA";
const SBC_INSTRUCTION: &str = "SBC";

pub const INSTRUCTION_SET: [Instruction; 27] = [
    Instruction {
        opcode: 0x06,
        mnemonic: ASL_INSTRUCTION,
        bytes: 2,
        clock_periods: 5,
        addressing_mode: AddressingMode::ZeroPage,
        execute: Cpu::asl_instruction,
    },
    Instruction {
        opcode: 0x0A,
        mnemonic: ASL_INSTRUCTION,
        bytes: 1,
        clock_periods: 2,
        addressing_mode: AddressingMode::Accumulator,
        execute: Cpu::asl_instruction,
    },
    Instruction {
        opcode: 0x21,
        mnemonic: AND_INSTRUCTION,
        bytes: 2,
        clock_periods: 6,
        addressing_mode: AddressingMode::IndirectX,
        execute: Cpu::and_instruction,
    },
    Instruction {
        opcode: 0x25,
        mnemonic: AND_INSTRUCTION,
        bytes: 2,
        clock_periods: 3,
        addressing_mode: AddressingMode::ZeroPage,
        execute: Cpu::and_instruction,
    },
    Instruction {
        opcode: 0x29,
        mnemonic: AND_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Immediate,
        execute: Cpu::and_instruction,
    },
    Instruction {
        opcode: 0x2d,
        mnemonic: AND_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::Absolute,
        execute: Cpu::and_instruction,
    },
    Instruction {
        opcode: 0x31,
        mnemonic: AND_INSTRUCTION,
        bytes: 2,
        clock_periods: 5,
        addressing_mode: AddressingMode::IndirectY,
        execute: Cpu::and_instruction,
    },
    Instruction {
        opcode: 0x35,
        mnemonic: AND_INSTRUCTION,
        bytes: 2,
        clock_periods: 4,
        addressing_mode: AddressingMode::ZeroPageX,
        execute: Cpu::and_instruction,
    },
    Instruction {
        opcode: 0x39,
        mnemonic: AND_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::AbsoluteY,
        execute: Cpu::and_instruction,
    },
    Instruction {
        opcode: 0x3d,
        mnemonic: AND_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::AbsoluteX,
        execute: Cpu::and_instruction,
    },
    Instruction {
        opcode: 0x61,
        mnemonic: ADC_INSTRUCTION,
        bytes: 2,
        clock_periods: 6,
        addressing_mode: AddressingMode::IndirectX,
        execute: Cpu::adc_instruction,
    },
    Instruction {
        opcode: 0x65,
        mnemonic: ADC_INSTRUCTION,
        bytes: 2,
        clock_periods: 3,
        addressing_mode: AddressingMode::ZeroPage,
        execute: Cpu::adc_instruction,
    },
    Instruction {
        opcode: 0x69,
        mnemonic: ADC_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Immediate,
        execute: Cpu::adc_instruction,
    },
    Instruction {
        opcode: 0x6D,
        mnemonic: ADC_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::Absolute,
        execute: Cpu::adc_instruction,
    },
    Instruction {
        opcode: 0x71,
        mnemonic: ADC_INSTRUCTION,
        bytes: 2,
        clock_periods: 5,
        addressing_mode: AddressingMode::IndirectY,
        execute: Cpu::adc_instruction,
    },
    Instruction {
        opcode: 0x75,
        mnemonic: ADC_INSTRUCTION,
        bytes: 2,
        clock_periods: 4,
        addressing_mode: AddressingMode::ZeroPageX,
        execute: Cpu::adc_instruction,
    },
    Instruction {
        opcode: 0x79,
        mnemonic: ADC_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::AbsoluteY,
        execute: Cpu::adc_instruction,
    },
    Instruction {
        opcode: 0x7D,
        mnemonic: ADC_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::AbsoluteX,
        execute: Cpu::adc_instruction,
    },
    Instruction {
        opcode: 0xA1,
        mnemonic: LDA_INSTRUCTION,
        bytes: 2,
        clock_periods: 6,
        addressing_mode: AddressingMode::IndirectX,
        execute: Cpu::lda_instruction,
    },
    Instruction {
        opcode: 0xA5,
        mnemonic: LDA_INSTRUCTION,
        bytes: 2,
        clock_periods: 3,
        addressing_mode: AddressingMode::ZeroPage,
        execute: Cpu::lda_instruction,
    },
    Instruction {
        opcode: 0xA9,
        mnemonic: LDA_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Immediate,
        execute: Cpu::lda_instruction,
    },
    Instruction {
        opcode: 0xAD,
        mnemonic: LDA_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::Absolute,
        execute: Cpu::lda_instruction,
    },
    Instruction {
        opcode: 0xB1,
        mnemonic: LDA_INSTRUCTION,
        bytes: 2,
        clock_periods: 5,
        addressing_mode: AddressingMode::IndirectY,
        execute: Cpu::lda_instruction,
    },
    Instruction {
        opcode: 0xB5,
        mnemonic: LDA_INSTRUCTION,
        bytes: 2,
        clock_periods: 4,
        addressing_mode: AddressingMode::ZeroPageX,
        execute: Cpu::lda_instruction,
    },
    Instruction {
        opcode: 0xB9,
        mnemonic: LDA_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::AbsoluteY,
        execute: Cpu::lda_instruction,
    },
    Instruction {
        opcode: 0xBD,
        mnemonic: LDA_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::AbsoluteX,
        execute: Cpu::lda_instruction,
    },
    Instruction {
        opcode: 0xE9,
        mnemonic: SBC_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Immediate,
        execute: Cpu::sbc_instruction,
    },
];
