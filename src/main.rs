mod cpu;

use crate::cpu::Cpu;

fn main() {
    let mut cpu = Cpu::new(0x8000);
    
    println!("{}", cpu.registers.to_string());
    println!("{}", cpu.memory.create_page_hexdump(0x00));

    cpu.power_up();
    
    println!("{}", cpu.registers.to_string());

    let _ = cpu.memory.read_raw_file_into_memory("/home/mark/rust/rust_6502_emulator/sample_code/lda-a9.o65", 0x8000);

    println!("{}", cpu.memory.create_page_hexdump(0x80));

    if let Some(line) = cpu.disassemble_opcode(0x8000) {
        println!("{}", line);
    }

    let _result = cpu.execute_opcode();
    
    println!("{}", cpu.registers.to_string());
}
