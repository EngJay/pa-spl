# PCB Artists I2C Sound Level Module

*Product sheet for the
[PCB Artists I2C Sound Level Module](https://pcbartists.com/product/i2c-decibel-sound-level-meter-module/).*

Accurately monitor sound level in decibel (dB SPL) with our low-power I2C
decibel sound level meter for ESP32, Arduino, Raspberry Pi, etc. [Contains
built-in microphone].

## Features

- High accuracy of ±2 dB SPL
- No external microphone needed – contains built-in MEMS mic
- Measurement range of 35 dB to 120 dB
- Frequency Range of 30 Hz to 8 kHz
- Easy to use – standard I2C interface
- Low power consumption, only 5mA @ 3.3V (measurement) and 100uA (sleep)
- Selectable response – A-weighted, C-weighted, Z-weighted
- Adjustable averaging time in the range of 10ms to 10,000 ms
  125ms (fast mode) and 1,000ms (slow mode) supported
- Threshold detection and interrupt
- 100-reading buffer to allow host MCU to sleep
- Stick to any surface using peel-off adhesive
- Low cost and small in size
- Audio Spectrum Analysis (Optional)
  - 0 – 8 kHz divided into 64 bands (125 Hz each), and
  - 0 – 8 kHz divided into 16 bands (500 Hz each).
  - Energy in each band is readable in dB SPL.

## Decibel Meter – Documentation

The I2C decibel meter module can be connected to any system with a 3.3V power
output and I2C bus for communicating with the sound sensor module.

- [Decibel Sensor – Interfacing Guide and Hardware Manual](https://pcbartists.com/product-documentation/decibel-meter-module-interfacing-guide/)

- Decibel Sensor configuration examples – using sleep, interrupt and other
  features (coming soon)
- [Decibel Sensor – Programming Guide and I2C Register Map](https://pcbartists.com/product-documentation/i2c-decibel-meter-programming-manual/)
  (includes Spectrum Analyzer registers)
- [Mounting guide for SMD type decibel sensor module](https://pcbartists.com/product-documentation/decibel-sensor-mounting-guidelines/)

## Example Projects and Source Code

Here are some sample projects and source code to help you get started quickly
with our decibel sensor module.

- [Using the dB SPL sensor with Raspberry Pi](https://pcbartists.com/product-documentation/accurate-raspberry-pi-decibel-meter/),
  includes example source code in C.

- [ESP32 Decibel Logger Demo with ThingsBoard](https://pcbartists.com/product-documentation/accurate-esp32-decibel-meter/),
  includes ESP32 Arduino sketch.
- [RP2040 or Pi Pico Sound Level Sensor Demo](https://pcbartists.com/product-documentation/pi-pico-sound-level-sensor/),
  includes MicroPython code.
- [Decibel Meter with Arduino UNO and 7-Segment Display](https://pcbartists.com/product-documentation/arduino-decibel-meter/),
  includes Arduino sketch.

## See the Module in Action

Watch How2Electronics’ YouTube video using an ESP32 devkit and OLED display to
show decibel values acquired from our sensor. At 0:40, you can see how the
speaker’s properties and the module’s A-weighting affects decibel readings based
on the sound frequency being measured.
