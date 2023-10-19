use super::Cpu;

#[derive(Copy, Clone)]
pub enum AddressingMode {
    Accumulator,
    Implied,
    Relative,
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
    pub set_program_counter: bool,
}

impl ExecutionReturnValues {
    pub fn new(instruction: Instruction, crossed_boundary: bool) -> ExecutionReturnValues {
        ExecutionReturnValues {
            bytes: instruction.bytes,
            clock_periods: match crossed_boundary {
                true => instruction.clock_periods + 1,
                false => instruction.clock_periods,
            },
            set_program_counter: instruction.sets_program_counter,
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
    pub sets_program_counter: bool,
    pub execute: fn(&mut Cpu, Instruction) -> ExecutionReturnValues,
}

const ADC_INSTRUCTION: &str = "ADC";
const AND_INSTRUCTION: &str = "AND";
const ASL_INSTRUCTION: &str = "ASL";
const BCC_INSTRUCTION: &str = "BCC";
const BCS_INSTRUCTION: &str = "BCS";
const BEQ_INSTRUCTION: &str = "BEQ";
const BIT_INSTRUCTION: &str = "BIT";
const BMI_INSTRUCTION: &str = "BMI";
const BNE_INSTRUCTION: &str = "BNE";
const BPL_INSTRUCTION: &str = "BPL";
const BRK_INSTRUCTION: &str = "BRK";
const BVC_INSTRUCTION: &str = "BVC";
const BVS_INSTRUCTION: &str = "BVS";
const CLC_INSTRUCTION: &str = "CLC";
const LDA_INSTRUCTION: &str = "LDA";
const SBC_INSTRUCTION: &str = "SBC";
const SEC_INSTRUCTION: &str = "SEC";

pub const INSTRUCTION_SET: [Instruction; 43] = [
    Instruction {
        opcode: 0x00,
        mnemonic: BRK_INSTRUCTION,
        bytes: 1,
        clock_periods: 7,
        addressing_mode: AddressingMode::Implied,
        sets_program_counter: false,
        execute: Cpu::brk_instruction,
    },
    Instruction {
        opcode: 0x06,
        mnemonic: ASL_INSTRUCTION,
        bytes: 2,
        clock_periods: 5,
        addressing_mode: AddressingMode::ZeroPage,
        sets_program_counter: false,
        execute: Cpu::asl_instruction,
    },
    Instruction {
        opcode: 0x0A,
        mnemonic: ASL_INSTRUCTION,
        bytes: 1,
        clock_periods: 2,
        addressing_mode: AddressingMode::Accumulator,
        sets_program_counter: false,
        execute: Cpu::asl_instruction,
    },
    Instruction {
        opcode: 0x0e,
        mnemonic: ASL_INSTRUCTION,
        bytes: 3,
        clock_periods: 6,
        addressing_mode: AddressingMode::Absolute,
        sets_program_counter: false,
        execute: Cpu::asl_instruction,
    },
    Instruction {
        opcode: 0x10,
        mnemonic: BPL_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Relative,
        sets_program_counter: true,
        execute: Cpu::bpl_instruction,
    },
    Instruction {
        opcode: 0x16,
        mnemonic: ASL_INSTRUCTION,
        bytes: 2,
        clock_periods: 6,
        addressing_mode: AddressingMode::ZeroPageX,
        sets_program_counter: false,
        execute: Cpu::asl_instruction,
    },
    Instruction {
        opcode: 0x18,
        mnemonic: CLC_INSTRUCTION,
        bytes: 1,
        clock_periods: 2,
        addressing_mode: AddressingMode::Implied,
        sets_program_counter: false,
        execute: Cpu::clc_instruction,
    },
    Instruction {
        opcode: 0x1e,
        mnemonic: ASL_INSTRUCTION,
        bytes: 3,
        clock_periods: 7,
        addressing_mode: AddressingMode::AbsoluteX,
        sets_program_counter: false,
        execute: Cpu::asl_instruction,
    },
    Instruction {
        opcode: 0x21,
        mnemonic: AND_INSTRUCTION,
        bytes: 2,
        clock_periods: 6,
        addressing_mode: AddressingMode::IndirectX,
        sets_program_counter: false,
        execute: Cpu::and_instruction,
    },
    Instruction {
        opcode: 0x24,
        mnemonic: BIT_INSTRUCTION,
        bytes: 2,
        clock_periods: 3,
        addressing_mode: AddressingMode::ZeroPage,
        sets_program_counter: false,
        execute: Cpu::bit_instruction,
    },
    Instruction {
        opcode: 0x25,
        mnemonic: AND_INSTRUCTION,
        bytes: 2,
        clock_periods: 3,
        addressing_mode: AddressingMode::ZeroPage,
        sets_program_counter: false,
        execute: Cpu::and_instruction,
    },
    Instruction {
        opcode: 0x29,
        mnemonic: AND_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Immediate,
        sets_program_counter: false,
        execute: Cpu::and_instruction,
    },
    Instruction {
        opcode: 0x2c,
        mnemonic: BIT_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::Absolute,
        sets_program_counter: false,
        execute: Cpu::bit_instruction,
    },
    Instruction {
        opcode: 0x2d,
        mnemonic: AND_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::Absolute,
        sets_program_counter: false,
        execute: Cpu::and_instruction,
    },
    Instruction {
        opcode: 0x30,
        mnemonic: BMI_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Relative,
        sets_program_counter: true,
        execute: Cpu::bmi_instruction,
    },
    Instruction {
        opcode: 0x31,
        mnemonic: AND_INSTRUCTION,
        bytes: 2,
        clock_periods: 5,
        addressing_mode: AddressingMode::IndirectY,
        sets_program_counter: false,
        execute: Cpu::and_instruction,
    },
    Instruction {
        opcode: 0x35,
        mnemonic: AND_INSTRUCTION,
        bytes: 2,
        clock_periods: 4,
        addressing_mode: AddressingMode::ZeroPageX,
        sets_program_counter: false,
        execute: Cpu::and_instruction,
    },
    Instruction {
        opcode: 0x38,
        mnemonic: SEC_INSTRUCTION,
        bytes: 1,
        clock_periods: 2,
        addressing_mode: AddressingMode::Implied,
        sets_program_counter: false,
        execute: Cpu::sec_instruction,
    },
    Instruction {
        opcode: 0x39,
        mnemonic: AND_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::AbsoluteY,
        sets_program_counter: false,
        execute: Cpu::and_instruction,
    },
    Instruction {
        opcode: 0x3d,
        mnemonic: AND_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::AbsoluteX,
        sets_program_counter: false,
        execute: Cpu::and_instruction,
    },
    Instruction {
        opcode: 0x50,
        mnemonic: BVC_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Relative,
        sets_program_counter: true,
        execute: Cpu::bvc_instruction,
    },
    Instruction {
        opcode: 0x61,
        mnemonic: ADC_INSTRUCTION,
        bytes: 2,
        clock_periods: 6,
        addressing_mode: AddressingMode::IndirectX,
        sets_program_counter: false,
        execute: Cpu::adc_instruction,
    },
    Instruction {
        opcode: 0x65,
        mnemonic: ADC_INSTRUCTION,
        bytes: 2,
        clock_periods: 3,
        addressing_mode: AddressingMode::ZeroPage,
        sets_program_counter: false,
        execute: Cpu::adc_instruction,
    },
    Instruction {
        opcode: 0x69,
        mnemonic: ADC_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Immediate,
        sets_program_counter: false,
        execute: Cpu::adc_instruction,
    },
    Instruction {
        opcode: 0x6D,
        mnemonic: ADC_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::Absolute,
        sets_program_counter: false,
        execute: Cpu::adc_instruction,
    },
    Instruction {
        opcode: 0x70,
        mnemonic: BVS_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Relative,
        sets_program_counter: true,
        execute: Cpu::bvs_instruction,
    },
    Instruction {
        opcode: 0x71,
        mnemonic: ADC_INSTRUCTION,
        bytes: 2,
        clock_periods: 5,
        addressing_mode: AddressingMode::IndirectY,
        sets_program_counter: false,
        execute: Cpu::adc_instruction,
    },
    Instruction {
        opcode: 0x75,
        mnemonic: ADC_INSTRUCTION,
        bytes: 2,
        clock_periods: 4,
        addressing_mode: AddressingMode::ZeroPageX,
        sets_program_counter: false,
        execute: Cpu::adc_instruction,
    },
    Instruction {
        opcode: 0x79,
        mnemonic: ADC_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::AbsoluteY,
        sets_program_counter: false,
        execute: Cpu::adc_instruction,
    },
    Instruction {
        opcode: 0x7D,
        mnemonic: ADC_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::AbsoluteX,
        sets_program_counter: false,
        execute: Cpu::adc_instruction,
    },
    Instruction {
        opcode: 0x90,
        mnemonic: BCC_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Relative,
        sets_program_counter: true,
        execute: Cpu::bcc_instruction,
    },
    Instruction {
        opcode: 0xA1,
        mnemonic: LDA_INSTRUCTION,
        bytes: 2,
        clock_periods: 6,
        addressing_mode: AddressingMode::IndirectX,
        sets_program_counter: false,
        execute: Cpu::lda_instruction,
    },
    Instruction {
        opcode: 0xA5,
        mnemonic: LDA_INSTRUCTION,
        bytes: 2,
        clock_periods: 3,
        addressing_mode: AddressingMode::ZeroPage,
        sets_program_counter: false,
        execute: Cpu::lda_instruction,
    },
    Instruction {
        opcode: 0xA9,
        mnemonic: LDA_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Immediate,
        sets_program_counter: false,
        execute: Cpu::lda_instruction,
    },
    Instruction {
        opcode: 0xAD,
        mnemonic: LDA_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::Absolute,
        sets_program_counter: false,
        execute: Cpu::lda_instruction,
    },
    Instruction {
        opcode: 0xB0,
        mnemonic: BCS_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Relative,
        sets_program_counter: true,
        execute: Cpu::bcs_instruction,
    },
    Instruction {
        opcode: 0xB1,
        mnemonic: LDA_INSTRUCTION,
        bytes: 2,
        clock_periods: 5,
        addressing_mode: AddressingMode::IndirectY,
        sets_program_counter: false,
        execute: Cpu::lda_instruction,
    },
    Instruction {
        opcode: 0xB5,
        mnemonic: LDA_INSTRUCTION,
        bytes: 2,
        clock_periods: 4,
        addressing_mode: AddressingMode::ZeroPageX,
        sets_program_counter: false,
        execute: Cpu::lda_instruction,
    },
    Instruction {
        opcode: 0xB9,
        mnemonic: LDA_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::AbsoluteY,
        sets_program_counter: false,
        execute: Cpu::lda_instruction,
    },
    Instruction {
        opcode: 0xBD,
        mnemonic: LDA_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::AbsoluteX,
        sets_program_counter: false,
        execute: Cpu::lda_instruction,
    },
    Instruction {
        opcode: 0xD0,
        mnemonic: BNE_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Relative,
        sets_program_counter: true,
        execute: Cpu::bne_instruction,
    },
    Instruction {
        opcode: 0xE9,
        mnemonic: SBC_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Immediate,
        sets_program_counter: false,
        execute: Cpu::sbc_instruction,
    },
    Instruction {
        opcode: 0xF0,
        mnemonic: BEQ_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Relative,
        sets_program_counter: true,
        execute: Cpu::beq_instruction,
    },
];
