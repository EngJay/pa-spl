# PCB Artists SPL Module Example

*A minimal example using a STM32F3 Discovery development board to read the
SPL from a PCB Artists SPL Module.*

## Building and Running

GDB and OpenOCD have been configured in the cargo config, so the example can be
built, flashed, and run using the Cargo run command as long as GDB and OpenOCD
are installed.

Hardware setup:

- Connect the V3.3 and ground pins of the sensor to matching pins on the dev
  board.
- Connect the SCL and SDA pins of the sensor to pins PB6 and PB7, respectively,
  on the dev board.
- Connect the TX and RX pins of your TTL-to-USB converter to pins PC11 and PC10,
  respectively, and connect the ground pin to a matching pin on the board.

Then, open a shell and run OpenOCD, to which a similar response as shown here
should be received if the board is connected with the onboard ST-Link debugger.

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

The gdb configuration sets some breakpoints by default, so it will likely be
necessary to continue past a few breakpoints to reach the main loop. That should
look simialr to this.

```bash
(gdb) continue
Continuing.

Breakpoint 4, example_read_decibel_value::__cortex_m_rt_main_trampoline ()
    at src/main.rs:49
49      #[entry]
(gdb) continue
Continuing.
```

Once running in the main loop, the SPL will be output via UART4 at 2 Hz. With a
TTL-to-USB converter, the output can be read into a shell. This output was
received using
[PySerial's Miniterm](https://pyserial.readthedocs.io/en/latest/tools.html#module-serial.tools.miniterm).

```bash
python -m serial.tools.miniterm /dev/cu.usbserial-A10KGFJI 115200
SPL: 51
```
