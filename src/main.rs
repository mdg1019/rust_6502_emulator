mod cpu;

use crate::cpu::Cpu;

fn main() {
    let mut cpu = Cpu::new(0x8000);

    println!("{}", cpu.registers.to_string());
    println!("{}", cpu.memory.create_page_hexdump(0x00));

    cpu.power_up();

    println!("{}", cpu.registers.to_string());

    let length = cpu.memory.read_raw_file_into_memory(
        "/home/mark/rust/rust_6502_emulator/sample_code/snippet.o65",
        0x8000,
    );

    println!("{}", cpu.memory.create_page_hexdump(0x80));

    let address: usize = 0x8000;
    let mut offset: usize = 0x0000;

    while offset < length {
        if let Some((line, bytes)) = cpu.disassemble_opcode(address + offset) {
            println!("{}", line);
            offset += bytes as usize;
        } else {
            break;
        }
    }

    let _result = cpu.execute_opcode();

    println!("{}", cpu.registers.to_string());
}
