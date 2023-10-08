mod cpu;

use crate::cpu::Cpu;

fn main() {
    let mut cpu = Cpu::new();
    
    println!("{}", cpu.registers.to_string());

    println!("{}", cpu.memory.get_8_bit_value(0x0000));

    cpu.memory.set_8_bit_value(0x0000, 0xff);

    println!("{}", cpu.memory.get_8_bit_value(0x0000));


}
