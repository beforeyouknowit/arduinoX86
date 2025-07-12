# ArduinoX86
![arduino8088_pcb](/images/render_v1_1.png)

### About ArduinoX86
I've written a blog article that gives an overview of this project and how it is used.

https://martypc.blogspot.com/2023/06/hardware-validating-emulator.html

This project was originally called Arduino8088, but then I expanded it to support the 8086, 80186, 80286, with plans for the 80386, so it is now named the suitably generic ArduinoX86.

### Description

This project expands on the basic idea of controlling a CPU via a microcontroller's GPIO pins to clock the CPU and read and write control and data signals.
This can be used to validate an emulator's accuracy, but also as a general method of exploring the operation of a CPU's instructions and timings.
This project specifically targets early Intel and compatible CPUs, such as the 8088, 8086, NEC V20, V30, and specific variants of the the Intel 80186. Future support for the 80286 is planned.

Where it differs from existing Raspberry Pi-based projects is that it uses an Arduino Giga for the expanded number of GPIO pins available. The Giga has enough GPIO to operate 808X-compatible CPUs in Maximum mode. This enables several useful signals to be read such as the QS0 & QS1 processor instruction queue status lines, which give us more insight into the internal state of the CPU. We can also enable inputs such as READY, NMI, INTR, and TEST, so we can execute interrupts, NMIs, emulate wait states, and potentially even simulate FPU and DMA operations.

I have been using this project to validate the cycle-accuracy of my PC emulator, [MartyPC](https://github.com/dbalsom/martypc), and produce CPU test suites for emulators, covering the [8088](https://github.com/singleStepTests/8088/), [V20](https://github.com/SingleStepTests/v20) and [8086](https://github.com/singleStepTests/8086/) to date. 

This project has three main components:
  - The CPU server software, which runs on the Arduino, and establishes a serial protocol to reset the CPU, load and store register state, as well as interactively operate the CPU if desired;
  - The CPU client software, which runs on your computer and controls the CPU after the CPU server has set up initial register state as desired, or uploads an entire program to be run automatically with register state collected at the end of execution;
  - The CPU socket PCB HAT(s), which seat on top of the Arduino, and provide a physical interface between the CPU and GPIO lines.

The original "Arduino8088" project utilized an Arduino MEGA, but this board is now considered deprecated for this project. It is painfully slow and limited to a serial UART instead of fast native USB communication. As of the current release, an [Arduino Due](https://store.arduino.cc/products/arduino-due), or preferably, an [Arduino GIGA](https://store.arduino.cc/products/giga-r1-wifi) should be used instead. A GIGA is required to operate 5V CPUs such as older 80186's and the 80286.  Despite Arduno's official warnings, the STM32 GPIO in the Giga is 5V tolerant (with a few exceptions).  You will likely void your warranty and I take no responsible for any damage to your Arduino that occurs attempting to use a 5V CPU with your Arduino.

### 8088,8086,V20,V30 Support

Revision 1.1 of the 8088 HAT supports the 8086 and NEC V30 by adding a connection for the 8086's BHE pin. 

The Due can operate the 8088, 8086, V20 and V30, despite these chips being 5V. The current 8088 hat feeds those CPUs 3.3V which they tolerate quite well at low clock speeds. 

The GIGA can operate the 8088, 8086, V20 and V30 directly at 5V if desired. The GIGA requires the 3V supply pin to be cut off the HAT's pin headers and external power supplied due to the lower available overall power budget. 

> [!WARNING]  
> Do not attempt to use the 8088 hat with a GIGA without using external power, or you may damage your Arduino.

### 80186 Support

The ArduinoX86 cpu_server sketch supports operation of a low-voltage 80186 on an Arduino Due. See the BOM section below for compatible parts. 
There is no HAT yet for the 80186, you will need to use a breakout board and connect directly to the Arduino Due's headers. See [Using an 80186](https://github.com/dbalsom/arduinoX86/wiki/Using-an-80186) for more information.

<img width="349" height="620" alt="image" src="https://github.com/user-attachments/assets/c7e5fa0e-8c6f-4119-a0f4-7b936193ac5e" />

### 80286 Support

Support for the 80286 has been added in the [80286 branch](https://github.com/dbalsom/arduinoX86/tree/80286) although this branch currently only supports the GIGA. 

<img width="2000" height="1125" alt="image" src="https://github.com/user-attachments/assets/2d5ffa03-ebcf-48c8-bfe2-1574fdb0de2b" />

A HAT for the 80286 for the Arduino GIGA is in the design phase. This HAT will have a DIP socket for a 287 for future expansion.

<img width="848" height="429" alt="image" src="https://github.com/user-attachments/assets/246e57b2-6c98-4540-a481-ff986699deae" />

### 80386 Support

A HAT for the 80386EX CPU is in the design phase.

<img width="889" height="498" alt="image" src="https://github.com/user-attachments/assets/769f2491-1e64-4626-9b32-65ec4cadb5c1" />

## Can the CPU be clocked fast enough with an Arduino?

In short, no. We are well past the published minimum cycle times when executing programs via a serial protocol, cycle by cycle. NMOS process chips will fail to function in strange ways when clocked below this threshold. The issue is "dynamic logic" - logic gates that lose their state if not refrehsed electrically within a frequent enough interval. To function with ArduinoX86 a CMOS process CPU with a fully static core should be employed. These chips can be clocked down to 0Hz and still function. A list of compatible CPUs is provided in the BOM below.

## Credits

Inspired by and borrows from the Pi8088 validator created by Andreas Jonsson as part of the VirtualXT project:

https://github.com/andreas-jonsson/virtualxt/tree/develop/tools/validator/pi8088

A very similar project is homebrew8088's Raspberry Pi Hat:

https://github.com/homebrew8088/pi86

## To use

If you are using an 8088, 8086, V20 or V30, and you don't want to order and build the PCB, connect the GPIO pins to the CPU on a breadboard as specified in the KiCad project schematic.

If you are using an 80186, see the [wiki](https://github.com/dbalsom/arduinoX86/wiki/Using-an-80186) for connection instructions.

The main Arduino program, 'cpu_server', operates a simple binary serial protocol to allow a client running on your PC to execute operations on the CPU and read and write data, status and control signals. This is designed for integration with an emulator or instruction test generator.

An example application for cpu_server is provided, written in Rust, in the /crates/exec_program directory. It demonstrates how to upload arbitrary code to the ArduinoX86 and display cycle traces. The client will emulate the entire address space and set up a basic IVT.

## 8088 (+8086/V20/V30) HAT
![pcb_shield50](/images/pcb_v1_1.png)

KiCad project files for the 8088 HAT PCB are supplied. 

Version 1.1 adds a connection for the 8086's BHE pin, which is used by the 8086/V30/80186/80286 to drive the upper half of the 16-bit data bus.

> [!WARNING]  
> Please read all the notes in the next section before ordering/assembling parts. Failure to heed warnings will cause damage to your Arduino.

> [!CAUTION]
> Depending on the CPU used and the presence of an 8288 support chip, you may exceed the rated 3V power supply of the Arduino Due.
> If your Arduino immediately turns off or does not turn on at all, you are likely triggering the overcurrent protection.
> 
> You can cut off the 3V supply lead from the header socket and provide external power from the top of the HAT.
>
> DO NOT connect external power to the 3V header without cutting the pin to the Arduino! Feeding current back into 3V OUT can damage your Arduino.

# BOM
- A compatible CPU:
  - For 8088, I recommend a CMOS variant such as a Harris or Oki 80C88. CMOS versions of the 8088 and 8086 were made by a number of manufacturers; all of them should work.
  - For 8086, Intel directly produced an 80C86 which works, so you might as well use it. The Harris 80C86 has also been tested but has minor differences. 
  - The NEC V20 and V30 were CMOS process CPUs and work as well.
  - Some NMOS process CPUs may work - the AMD D8088 will work, but the AMD D8086-2 will not. The original Intel 8088 (C)1978 will not function properly.
  - For 80186, I recommend the Intel 80L186EB. See [Using an 80186](https://github.com/dbalsom/arduinoX86/wiki/Using-an-80186) for more information.

  - For 80286, a level-shifting HAT is required as no 3V 286 was produced. A CMOS variant such as the Harris 80C286 is required. The CPU should be a PLCC68 form factor.
    The 286 HAT is still in the design phase.

> [!TIP]
> Beware of counterfeits on eBay and other online vendors. A legitimate chip will not look shiny and new with perfect printing on it. At best, you may get a functional
> CPU that has been resurfaced and reprinted, but there's not telling what you're actualy testing. 

- Optional: If using an 8088 or 8086, you can socket an Intel 8288 or OKI 82C88 Bus Controller.
  A CMOS version it not necessarily required, but they use less power.
  
  The 8288 is easily emulated but its ALE output is used to drive the LED on the Arduino8088 HAT.
  Change the `EMULATE_8288` define to 0 if using a physical 8288.

- A set of Arduino stacking headers (also usable with Due) 
https://www.amazon.com/Treedix-Stacking-Headers-Stackable-Compatible/dp/B08G4FGBPQ

- A DIP-40 and (optionally) DIP-20 socket
  - Optional but highly recommended: A ZIF socket such as [https://www.amazon.com/-/en/gp/product/B00B886OZI](https://www.amazon.com/-/en/gp/product/B00B886OZI)
    to avoid breaking pins on repeated insertions

- (2x) 0805 0.047uf bypass capacitors
  https://www.mouser.com/ProductDetail/80-C0805C473KARAUTO

- Optional: A 12mm, active buzzer with 7.6mm pin spacing. 

  - For Due: A 3V piezoelectric, low power buzzer <= 6mA
    https://www.mouser.com/ProductDetail/Mallory-Sonalert/PK-11N40PQ?qs=SXHtpsd1MbZ%252B7jeUyAAOVA%3D%3D

> [!WARNING]  
> Do not connect an electromagnetic buzzer. The Due has insufficient GPIO current supply.

- (2x) 750Ohm resistors (for LEDs)
  https://www.mouser.com/ProductDetail/667-ERA-6AED751V

- (2x) Any 0805 ~2V LED of your choice with 1.8-1.9mA forward current
  - https://www.mouser.com/ProductDetail/604-APTD2012LCGCK (Green)
  - https://www.mouser.com/ProductDetail/604-APT2012LSECKJ4RV (Orange)
 
- RS232 board for debug output - choose gender based on your desired cabling
  - https://www.amazon.com/Ultra-Compact-RS232-Converter-1Mbps/dp/B074BMLM11 (male)
  - https://www.amazon.com/Ultra-Compact-RS232-Converter-Female/dp/B074BTGLJN (female)
    
> [!WARNING]  
> DO NOT connect 5V to the RS232 board on the Arduino Due. You will feed 5V into the RX GPIO pin and cause irreperable damage.

# Project Structure

## /asm

Assembly language files, intended to be assembled with NASM. To execute code on the ArduinoX86, one must supply two
binary files, one containing the program to be executed, and one containing the register values to load onto the CPU
before program execution. 

## /crates/ard808x_client

A library crate that implements a client for the ArduinoX86's serial protocol.

## /crates/ard808x_cpu

A library crate built on top of the `ard808x_client` crate, this provides a `RemoteCpu` struct that models CPU state 
and can execute programs.

## /crates/exec_program

A binary implementing an interface for the `ard808x_cpu` crate that will load a provided register state binary and 
execute the specified program binary.

## /pcb

Contains the KiCad project files and Gerber files for the Arduino8088 PCB.

## /sketches/cpu_server

The main Arduino sketch for ArduinoX86. Implements a server for a serial protocol enabling remote control of a 16-bit
Intel CPU on the Arduino Due.

## /sketches/run_program

An older sketch that can execute a program directly on the Arduino MEGA. Supports the 8088 only.
