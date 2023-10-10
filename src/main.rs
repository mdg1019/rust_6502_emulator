mod cpu;

use crate::cpu::Cpu;

fn main() {
    let mut cpu = Cpu::new(0x8000);
    
    println!("{}", cpu.registers.to_string());
    println!("{}", cpu.memory.create_page_hexdump(0x00));

    cpu.power_up();
    
    println!("{}", cpu.registers.to_string());


}
