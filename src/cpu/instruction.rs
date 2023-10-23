use super::Cpu;

#[derive(Copy, Clone)]
pub enum AddressingMode {
    Accumulator,
    Implied,
    Relative,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX, // (Zero-Page,X)
    IndirectY, // (Zero-Page),Ys
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
const CLD_INSTRUCTION: &str = "CLD";
const CLI_INSTRUCTION: &str = "CLI";
const CLV_INSTRUCTION: &str = "CLV";
const CMP_INSTRUCTION: &str = "CMP";
const CPX_INSTRUCTION: &str = "CPX";
const CPY_INSTRUCTION: &str = "CPY";
const DEC_INSTRUCTION: &str = "DEC";
const DEX_INSTRUCTION: &str = "DEX";
const DEY_INSTRUCTION: &str = "DEY";
const EOR_INSTRUCTION: &str = "EOR";
const INC_INSTRUCTION: &str = "INC";
const INX_INSTRUCTION: &str = "INX";
const INY_INSTRUCTION: &str = "INY";
const JMP_INSTRUCTION: &str = "JMP";
const JSR_INSTRUCTION: &str = "JSR";
const LDA_INSTRUCTION: &str = "LDA";
const LDX_INSTRUCTION: &str = "LDX";
const LDY_INSTRUCTION: &str = "LDY";
const LSR_INSTRUCTION: &str = "LSR";
const NOP_INSTRUCTION: &str = "NOP";
const ORA_INSTRUCTION: &str = "ORA";
const PHA_INSTRUCTION: &str = "PHA";
const SBC_INSTRUCTION: &str = "SBC";
const SEC_INSTRUCTION: &str = "SEC";

pub const INSTRUCTION_SET: [Instruction; 108] = [
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
        opcode: 0x01,
        mnemonic: ORA_INSTRUCTION,
        bytes: 2,
        clock_periods: 6,
        addressing_mode: AddressingMode::IndirectX,
        sets_program_counter: false,
        execute: Cpu::ora_instruction,
    },
    Instruction {
        opcode: 0x05,
        mnemonic: ORA_INSTRUCTION,
        bytes: 2,
        clock_periods: 3,
        addressing_mode: AddressingMode::ZeroPage,
        sets_program_counter: false,
        execute: Cpu::ora_instruction,
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
        opcode: 0x09,
        mnemonic: ORA_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Immediate,
        sets_program_counter: false,
        execute: Cpu::ora_instruction,
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
        opcode: 0x0D,
        mnemonic: ORA_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::Absolute,
        sets_program_counter: false,
        execute: Cpu::ora_instruction,
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
        opcode: 0x11,
        mnemonic: ORA_INSTRUCTION,
        bytes: 2,
        clock_periods: 5,
        addressing_mode: AddressingMode::IndirectY,
        sets_program_counter: false,
        execute: Cpu::ora_instruction,
    },
    Instruction {
        opcode: 0x15,
        mnemonic: ORA_INSTRUCTION,
        bytes: 2,
        clock_periods: 4,
        addressing_mode: AddressingMode::ZeroPageX,
        sets_program_counter: false,
        execute: Cpu::ora_instruction,
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
        opcode: 0x19,
        mnemonic: ORA_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::AbsoluteY,
        sets_program_counter: false,
        execute: Cpu::ora_instruction,
    },
    Instruction {
        opcode: 0x1D,
        mnemonic: ORA_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::AbsoluteX,
        sets_program_counter: false,
        execute: Cpu::ora_instruction,
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
        opcode: 0x20,
        mnemonic: JSR_INSTRUCTION,
        bytes: 3,
        clock_periods: 6,
        addressing_mode: AddressingMode::Absolute,
        sets_program_counter: true,
        execute: Cpu::jsr_instruction,
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
        opcode: 0x41,
        mnemonic: EOR_INSTRUCTION,
        bytes: 2,
        clock_periods: 6,
        addressing_mode: AddressingMode::IndirectX,
        sets_program_counter: false,
        execute: Cpu::eor_instruction,
    },
    Instruction {
        opcode: 0x45,
        mnemonic: EOR_INSTRUCTION,
        bytes: 2,
        clock_periods: 3,
        addressing_mode: AddressingMode::ZeroPage,
        sets_program_counter: false,
        execute: Cpu::eor_instruction,
    },
    Instruction {
        opcode: 0x46,
        mnemonic: LSR_INSTRUCTION,
        bytes: 2,
        clock_periods: 5,
        addressing_mode: AddressingMode::ZeroPage,
        sets_program_counter: false,
        execute: Cpu::lsr_instruction,
    },
    Instruction {
        opcode: 0x48,
        mnemonic: PHA_INSTRUCTION,
        bytes: 1,
        clock_periods: 3,
        addressing_mode: AddressingMode::Implied,
        sets_program_counter: false,
        execute: Cpu::pha_instruction,
    },
    Instruction {
        opcode: 0x49,
        mnemonic: EOR_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Immediate,
        sets_program_counter: false,
        execute: Cpu::eor_instruction,
    },
    Instruction {
        opcode: 0x4A,
        mnemonic: LSR_INSTRUCTION,
        bytes: 1,
        clock_periods: 2,
        addressing_mode: AddressingMode::Accumulator,
        sets_program_counter: false,
        execute: Cpu::lsr_instruction,
    },
    Instruction {
        opcode: 0x4C,
        mnemonic: JMP_INSTRUCTION,
        bytes: 3,
        clock_periods: 3,
        addressing_mode: AddressingMode::Absolute,
        sets_program_counter: true,
        execute: Cpu::jmp_instruction,
    },
    Instruction {
        opcode: 0x4D,
        mnemonic: EOR_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::Absolute,
        sets_program_counter: false,
        execute: Cpu::eor_instruction,
    },
    Instruction {
        opcode: 0x4E,
        mnemonic: LSR_INSTRUCTION,
        bytes: 3,
        clock_periods: 6,
        addressing_mode: AddressingMode::Absolute,
        sets_program_counter: false,
        execute: Cpu::lsr_instruction,
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
        opcode: 0x51,
        mnemonic: EOR_INSTRUCTION,
        bytes: 2,
        clock_periods: 5,
        addressing_mode: AddressingMode::IndirectY,
        sets_program_counter: false,
        execute: Cpu::eor_instruction,
    },
    Instruction {
        opcode: 0x55,
        mnemonic: EOR_INSTRUCTION,
        bytes: 2,
        clock_periods: 4,
        addressing_mode: AddressingMode::ZeroPageX,
        sets_program_counter: false,
        execute: Cpu::eor_instruction,
    },
    Instruction {
        opcode: 0x56,
        mnemonic: LSR_INSTRUCTION,
        bytes: 2,
        clock_periods: 6,
        addressing_mode: AddressingMode::ZeroPageX,
        sets_program_counter: false,
        execute: Cpu::lsr_instruction,
    },
    Instruction {
        opcode: 0x58,
        mnemonic: CLI_INSTRUCTION,
        bytes: 1,
        clock_periods: 2,
        addressing_mode: AddressingMode::Implied,
        sets_program_counter: false,
        execute: Cpu::cli_instruction,
    },
    Instruction {
        opcode: 0x59,
        mnemonic: EOR_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::AbsoluteY,
        sets_program_counter: false,
        execute: Cpu::eor_instruction,
    },
    Instruction {
        opcode: 0x5D,
        mnemonic: EOR_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::AbsoluteX,
        sets_program_counter: false,
        execute: Cpu::eor_instruction,
    },
    Instruction {
        opcode: 0x5E,
        mnemonic: LSR_INSTRUCTION,
        bytes: 3,
        clock_periods: 7,
        addressing_mode: AddressingMode::AbsoluteX,
        sets_program_counter: false,
        execute: Cpu::lsr_instruction,
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
        opcode: 0x6C,
        mnemonic: JMP_INSTRUCTION,
        bytes: 3,
        clock_periods: 5,
        addressing_mode: AddressingMode::Indirect,
        sets_program_counter: true,
        execute: Cpu::jmp_instruction,
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
        opcode: 0x88,
        mnemonic: DEY_INSTRUCTION,
        bytes: 1,
        clock_periods: 2,
        addressing_mode: AddressingMode::Implied,
        sets_program_counter: false,
        execute: Cpu::dey_instruction,
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
        opcode: 0xA0,
        mnemonic: LDY_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Immediate,
        sets_program_counter: false,
        execute: Cpu::ldy_instruction,
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
        opcode: 0xA2,
        mnemonic: LDX_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Immediate,
        sets_program_counter: false,
        execute: Cpu::ldx_instruction,
    },
    Instruction {
        opcode: 0xA4,
        mnemonic: LDY_INSTRUCTION,
        bytes: 2,
        clock_periods: 3,
        addressing_mode: AddressingMode::ZeroPage,
        sets_program_counter: false,
        execute: Cpu::ldy_instruction,
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
        opcode: 0xA6,
        mnemonic: LDX_INSTRUCTION,
        bytes: 2,
        clock_periods: 3,
        addressing_mode: AddressingMode::ZeroPage,
        sets_program_counter: false,
        execute: Cpu::ldx_instruction,
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
        opcode: 0xAC,
        mnemonic: LDY_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::Absolute,
        sets_program_counter: false,
        execute: Cpu::ldy_instruction,
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
        opcode: 0xAE,
        mnemonic: LDX_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::Absolute,
        sets_program_counter: false,
        execute: Cpu::ldx_instruction,
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
        opcode: 0xB4,
        mnemonic: LDY_INSTRUCTION,
        bytes: 2,
        clock_periods: 4,
        addressing_mode: AddressingMode::ZeroPageX,
        sets_program_counter: false,
        execute: Cpu::ldy_instruction,
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
        opcode: 0xB6,
        mnemonic: LDX_INSTRUCTION,
        bytes: 2,
        clock_periods: 4,
        addressing_mode: AddressingMode::ZeroPageY,
        sets_program_counter: false,
        execute: Cpu::ldx_instruction,
    },
    Instruction {
        opcode: 0xB8,
        mnemonic: CLV_INSTRUCTION,
        bytes: 1,
        clock_periods: 2,
        addressing_mode: AddressingMode::Implied,
        sets_program_counter: false,
        execute: Cpu::clv_instruction,
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
        opcode: 0xBC,
        mnemonic: LDY_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::AbsoluteX,
        sets_program_counter: false,
        execute: Cpu::ldy_instruction,
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
        opcode: 0xBE,
        mnemonic: LDX_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::AbsoluteY,
        sets_program_counter: false,
        execute: Cpu::ldx_instruction,
    },
    Instruction {
        opcode: 0xC0,
        mnemonic: CPY_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Immediate,
        sets_program_counter: false,
        execute: Cpu::cpy_instruction,
    },
    Instruction {
        opcode: 0xC1,
        mnemonic: CMP_INSTRUCTION,
        bytes: 2,
        clock_periods: 6,
        addressing_mode: AddressingMode::IndirectX,
        sets_program_counter: false,
        execute: Cpu::cmp_instruction,
    },
    Instruction {
        opcode: 0xC4,
        mnemonic: CPY_INSTRUCTION,
        bytes: 2,
        clock_periods: 3,
        addressing_mode: AddressingMode::ZeroPage,
        sets_program_counter: false,
        execute: Cpu::cpx_instruction,
    },
    Instruction {
        opcode: 0xC5,
        mnemonic: CMP_INSTRUCTION,
        bytes: 2,
        clock_periods: 3,
        addressing_mode: AddressingMode::ZeroPage,
        sets_program_counter: false,
        execute: Cpu::cmp_instruction,
    },
    Instruction {
        opcode: 0xC6,
        mnemonic: DEC_INSTRUCTION,
        bytes: 2,
        clock_periods: 5,
        addressing_mode: AddressingMode::ZeroPage,
        sets_program_counter: false,
        execute: Cpu::dec_instruction,
    },
    Instruction {
        opcode: 0xC8,
        mnemonic: INY_INSTRUCTION,
        bytes: 1,
        clock_periods: 2,
        addressing_mode: AddressingMode::Implied,
        sets_program_counter: false,
        execute: Cpu::iny_instruction,
    },
    Instruction {
        opcode: 0xC9,
        mnemonic: CMP_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Immediate,
        sets_program_counter: false,
        execute: Cpu::cmp_instruction,
    },
    Instruction {
        opcode: 0xCA,
        mnemonic: DEX_INSTRUCTION,
        bytes: 1,
        clock_periods: 2,
        addressing_mode: AddressingMode::Implied,
        sets_program_counter: false,
        execute: Cpu::dex_instruction,
    },
    Instruction {
        opcode: 0xCC,
        mnemonic: CPY_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::Absolute,
        sets_program_counter: false,
        execute: Cpu::cpx_instruction,
    },
    Instruction {
        opcode: 0xCD,
        mnemonic: CMP_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::Absolute,
        sets_program_counter: false,
        execute: Cpu::cmp_instruction,
    },
    Instruction {
        opcode: 0xCE,
        mnemonic: DEC_INSTRUCTION,
        bytes: 3,
        clock_periods: 6,
        addressing_mode: AddressingMode::Absolute,
        sets_program_counter: false,
        execute: Cpu::dec_instruction,
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
        opcode: 0xD1,
        mnemonic: CMP_INSTRUCTION,
        bytes: 2,
        clock_periods: 5,
        addressing_mode: AddressingMode::IndirectY,
        sets_program_counter: false,
        execute: Cpu::cmp_instruction,
    },
    Instruction {
        opcode: 0xD5,
        mnemonic: CMP_INSTRUCTION,
        bytes: 2,
        clock_periods: 4,
        addressing_mode: AddressingMode::ZeroPageX,
        sets_program_counter: false,
        execute: Cpu::cmp_instruction,
    },
    Instruction {
        opcode: 0xD6,
        mnemonic: DEC_INSTRUCTION,
        bytes: 2,
        clock_periods: 6,
        addressing_mode: AddressingMode::ZeroPageX,
        sets_program_counter: false,
        execute: Cpu::dec_instruction,
    },
    Instruction {
        opcode: 0xD8,
        mnemonic: CLD_INSTRUCTION,
        bytes: 1,
        clock_periods: 2,
        addressing_mode: AddressingMode::Implied,
        sets_program_counter: false,
        execute: Cpu::cld_instruction,
    },
    Instruction {
        opcode: 0xD9,
        mnemonic: CMP_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::AbsoluteY,
        sets_program_counter: false,
        execute: Cpu::cmp_instruction,
    },
    Instruction {
        opcode: 0xDD,
        mnemonic: CMP_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::AbsoluteX,
        sets_program_counter: false,
        execute: Cpu::cmp_instruction,
    },
    Instruction {
        opcode: 0xDE,
        mnemonic: DEC_INSTRUCTION,
        bytes: 3,
        clock_periods: 7,
        addressing_mode: AddressingMode::AbsoluteX,
        sets_program_counter: false,
        execute: Cpu::dec_instruction,
    },
    Instruction {
        opcode: 0xE0,
        mnemonic: CPX_INSTRUCTION,
        bytes: 2,
        clock_periods: 2,
        addressing_mode: AddressingMode::Immediate,
        sets_program_counter: false,
        execute: Cpu::cpx_instruction,
    },
    Instruction {
        opcode: 0xE4,
        mnemonic: CPX_INSTRUCTION,
        bytes: 2,
        clock_periods: 3,
        addressing_mode: AddressingMode::ZeroPage,
        sets_program_counter: false,
        execute: Cpu::cpx_instruction,
    },
    Instruction {
        opcode: 0xE6,
        mnemonic: INC_INSTRUCTION,
        bytes: 2,
        clock_periods: 5,
        addressing_mode: AddressingMode::ZeroPage,
        sets_program_counter: false,
        execute: Cpu::inc_instruction,
    },
    Instruction {
        opcode: 0xE8,
        mnemonic: INX_INSTRUCTION,
        bytes: 1,
        clock_periods: 2,
        addressing_mode: AddressingMode::Implied,
        sets_program_counter: false,
        execute: Cpu::inx_instruction,
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
        opcode: 0xEA,
        mnemonic: NOP_INSTRUCTION,
        bytes: 1,
        clock_periods: 2,
        addressing_mode: AddressingMode::Implied,
        sets_program_counter: false,
        execute: Cpu::nop_instruction,
    },
    Instruction {
        opcode: 0xEC,
        mnemonic: CPX_INSTRUCTION,
        bytes: 3,
        clock_periods: 4,
        addressing_mode: AddressingMode::Absolute,
        sets_program_counter: false,
        execute: Cpu::cpx_instruction,
    },
    Instruction {
        opcode: 0xEE,
        mnemonic: INC_INSTRUCTION,
        bytes: 3,
        clock_periods: 6,
        addressing_mode: AddressingMode::Absolute,
        sets_program_counter: false,
        execute: Cpu::inc_instruction,
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
    Instruction {
        opcode: 0xF6,
        mnemonic: INC_INSTRUCTION,
        bytes: 2,
        clock_periods: 6,
        addressing_mode: AddressingMode::ZeroPageX,
        sets_program_counter: false,
        execute: Cpu::inc_instruction,
    },
    Instruction {
        opcode: 0xFE,
        mnemonic: INC_INSTRUCTION,
        bytes: 3,
        clock_periods: 7,
        addressing_mode: AddressingMode::AbsoluteX,
        sets_program_counter: false,
        execute: Cpu::inc_instruction,
    },
];
