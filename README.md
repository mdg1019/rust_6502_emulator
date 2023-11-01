# rust_6502_eumulator
rust_6502_emulator is Rust implementation of a a 6502 emulator. I've been wanting to create something with Rust for a couple of years now and I've been wanting to write a microprocessor emulator for even longer. So the two desires came to fruition in this project.

It features a built-in debugger. Admittedly, the debugger is still a little crude. It could use some more functionality. My goal for the time being was to get something useful that I could use to perform emulation testing.

The project has many unit tests, which I believe helped eliminate a lot of potential problems.

To date, it successfully passes Klaus Dormann's functional and decimal tests: [https://github.com/Klaus2m5/6502_65C02_functional_tests](https://github.com/Klaus2m5/6502_65C02_functional_tests).

I used the CA65 versions of Klaus' tests: [https://github.com/amb5l/6502_65C02_functional_tests](https://github.com/amb5l/6502_65C02_functional_tests).


