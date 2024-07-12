# PCB Artists SPL Module Example

*A minimal example of using a STM32F3 Discovery development board to read the
SPL from a PCB Artists SPL Module and write it out via UART.*

## Building and Running

GDB and OpenOCD have been configured in the cargo config, so the example can be
built, flashed, and run by using the Cargo run command.

First, open a shell and run OpenOCD, to which a similar response should be
received if the board is connected with the onboard ST-Link debuggr.

```bash
openocd

Open On-Chip Debugger 0.12.0
Licensed under GNU GPL v2
For bug reports, read
        http://openocd.org/doc/doxygen/bugs.html
Info : auto-selecting first available session transport "hla_swd". To override use 'transport select <transport>'.
Info : The selected transport took over low-level target control. The results might differ compared to plain JTAG/SWD
Info : Listening on port 6666 for tcl connections
Info : Listening on port 4444 for telnet connections
Info : clock speed 1000 kHz
Info : STLINK V2J45M30 (API v2) VID:PID 0483:374B
Info : Target voltage: 2.912247
Info : [stm32f3x.cpu] Cortex-M4 r0p1 processor detected
Info : [stm32f3x.cpu] target has 6 breakpoints, 4 watchpoints
Info : starting gdb server for stm32f3x.cpu on 3333
Info : Listening on port 3333 for gdb connections
```

Open a second shell, then build, flash, and run the example via Cargo. After the
build of the example and initialization of gdb, it should stop at a gdb prompt.

```bash
cargo run

... lots of output ...

(gdb)
```

The gdp configuration sets some breakpoints by default, so it will likely be
necessary to continue past a few breakpoints to reach the main loop.

```bash
(gdb) continue
Continuing.

Breakpoint 4, example_read_decibel_value::__cortex_m_rt_main_trampoline ()
    at src/main.rs:49
49      #[entry]
(gdb) continue
Continuing.
```

Once running, the SPL will be printed out via UART4.

```bash
SPL: 51
SPL: 52
SPL: 53
SPL: 52
SPL: 52
SPL: 51
SPL: 52
SPL: 51
SPL: 52

... and so on...
```
