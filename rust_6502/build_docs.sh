#!/bin/bash
echo "<meta http-equiv=\"refresh\" content=\"0; url=rust_6502/target/doc/rust_6502/index.html\">" > ../index.html
cargo doc --open
