mod cpu;

use crate::cpu::Cpu;

fn main() {
    let mut cpu = Cpu::new();
    println!("{}", cpu.registers.to_string());
}
