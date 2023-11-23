# Rust 6502 Emulator

The API docs are [here](https://mdg1019.github.io/rust_6502_emulator/index.html).

This project implements a 6502 emulator in Rust. I've been wanting to create something with Rust for a couple of years now and I've been wanting to write a microprocessor emulator for even longer. So the two desires came to fruition in this project.

The emulator features a built-in debugger. Admittedly, the debugger is still a little crude. It could use some more functionality. My goal for the time being was to get something useful that I could use to perform emulation testing. It achieved that goal and I'll probably improve it in the future.

The project has many unit tests, which I believe helped eliminate a lot of potential problems.

To date, the emulator successfully passes Klaus Dormann's functional and decimal tests available [here](https://github.com/Klaus2m5/6502_65C02_functional_tests). I used the CA65 versions of Klaus' tests available [here](https://github.com/amb5l/6502_65C02_functional_tests).

I've done some profiling using [Samply](https://github.com/mstange/samply).

You must build the **rust_6502** project in release mode for the realtime clock emulation to work. Debug mode is fine for development, but you definitely want the speed optimizations available in release mode. You can build the **rust_6502** project in release mode with the following command line:

```
cargo build --release
```

To use the emulator, you must first instantiate a **Cpu** object by calling the **Cpu::new()** function as follows:

~~~rust
let mut cpu = Cpu::new(0x0400, 1_789_773.0);
~~~

As the above example shows, the **Cpu::new()** function takes two parameters. 

The first parameter is the **starting address** for the code to be executed. This **starting address** will be stored in the 6502 **reset vector** as an unsigned, 16-bit value starting at **0xFFFC**. In this example, the starting address of **0x0400** will be stored in the **reset vector**. Therefore, **0xFFFC** will be equal to **0x00** and **0xFFFD** will be equal to **0x04**.

The second parameter is the clock speed for the cpu. In this case, the cpu's clock speed will be set to 1.789773 MHz. 

Next, the **Cpu** object has to be powered up as follows:

~~~rust
cpu.power_up();
~~~

The **power_up()** method sets the **interrupt disable flag** to **true**, sets the **break flag** to **false**, sets the **stack pointer** to **0xFF**, and sets the **program counter** to the contents of the **reset vector**, which was previously set by the **Cpu::new()** function.

To execute a program without debugging, you use the **Cpu** object's **run()** method as follows:

~~~rust
cpu.run(None);
~~~

The **run()** method takes a single parameter, which expects an **Option<fn(&str) -> String>** function. If you pass it **None** as in the above example, the emulator will run wihtout debugging the code.

You could pass it a closure like in the following code to support debugging. This example allows the user to debug the code in the terminal. 

~~~rust
cpu.run(Some(|s: &str| {
    println!("{}", s);
    print!("Debug Command: ");

    io::Write::flush(&mut io::stdout()).expect("flush failed!");

    let mut buffer = String::new();

    io::stdin().read_line(&mut buffer).unwrap();

    buffer
}));
~~~

# Debugging Commands

The debugger supports the following commands:

|Command|Description|
|-------|-----------|
|?|**Help** - Displays a list of available debugger commands with a short description for each command.|
|B address|**(B)reakpoint** - Sets or deletes a breakpoint at the specified address. The address is specified as a 1 to 4 digit hexadecimal value.|
|D address|**(D)isplay** - Displays the next 16 bytes beginning at the specified address. The address is specified as a 1 to 4 digit hexadecimal value.|
|Q|**(Q)uit** - Stops the currently executing code and exits the **run()** method.|
|S|**(S)tep** - Executes the next opcode and pauses execution.|
|T|**(T)rap** - Enables or disables trapping. This is set to **true** by default. Trapping causes the cpu to monitor the **program counter** to see if it is equal to the last address that was just executed. If so, the code is in an infinite loop and is meaningful with some test suites like the ones I used to validate the emulator.|
|X|**E(x)ecute** - Runs the program starting at the location in the **program counter**.|
