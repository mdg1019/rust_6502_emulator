mod cpu;

use crate::cpu::Cpu;

fn main() {
    let mut cpu = Cpu::new();
    println!("A: {}", cpu.registers.a);
}
