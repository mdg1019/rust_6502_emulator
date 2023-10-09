mod cpu;

use crate::cpu::Cpu;

fn main() {
    let mut cpu = Cpu::new(0x8000);
    
    println!("{}", cpu.registers.to_string());

    cpu.power_up();
    
    println!("{}", cpu.registers.to_string());

}
