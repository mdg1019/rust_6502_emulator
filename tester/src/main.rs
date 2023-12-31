extern crate rust_6502;
use std::io;

use rust_6502::cpu::Cpu;

fn main() {

    println!("{}", std::env::current_dir().unwrap().to_str().unwrap());

    let mut cpu = Cpu::new(0x0400, 1_789_773.0);

    cpu.power_up();

    let _ = cpu
        .memory
        .read_raw_file_into_memory("../test_suites/6502_functional_test.bin", 0x0000);

    // let _ = cpu
    //     .memory
    //     .read_raw_file_into_memory("../test_suites/6502_decimal_test.bin", 0x0000);

    cpu.run(Some(|s: &str| {
        println!("{}", s);
        print!("Debug Command: ");

        io::Write::flush(&mut io::stdout()).expect("flush failed!");

        let mut buffer = String::new();

        io::stdin().read_line(&mut buffer).unwrap();

        buffer
    }));

    // cpu.run(None);
}
