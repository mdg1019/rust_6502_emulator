mod registers;
mod status_flags;

use crate::registers::Registers;

fn main() {
    let mut registers = Registers::new();
    println!("A: {}", registers.a);
}
