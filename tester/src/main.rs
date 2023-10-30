extern crate rust_6502;
use rust_6502::cpu::Cpu;

fn main() {
    let mut cpu = Cpu::new(0x0400, 1_789_773.0);
    cpu.power_up();

    println!("{}", cpu.registers.to_string());
    println!("{}", cpu.memory.create_page_hexdump(0x00));

    println!("{}", cpu.registers.to_string());

    let mut length = cpu.memory.read_raw_file_into_memory(
        "/home/mark/6502TestFiles/6502_functional_test.bin",
        0x0000,
    );

    println!("{}", cpu.memory.create_page_hexdump(0x04));

    let address: usize = 0x0400;

    print!("{}", cpu.disassemble_lines(address, 16));

    cpu.run(
        true,
        Some(|| { "Debugger input".to_string() }),
        Some(| s: String | { println!("{}", s); })
    );
}
