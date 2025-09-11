/*
    ArduinoX86 Copyright 2022-2025 Daniel Balsom
    https://github.com/dbalsom/arduinoX86

    Permission is hereby granted, free of charge, to any person obtaining a
    copy of this software and associated documentation files (the “Software”),
    to deal in the Software without restriction, including without limitation
    the rights to use, copy, modify, merge, publish, distribute, sublicense,
    and/or sell copies of the Software, and to permit persons to whom the
    Software is furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in
    all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER   
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
    FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
    DEALINGS IN THE SOFTWARE.
*/

// Functions for reading and writing the CPU bus.

#define WRITE_BIT(data, mask, set_macro, clear_macro) \
    do { if ((data) & (mask)) { set_macro; } else { clear_macro; } } while (0)


// Write a value to the CPU's data bus
void data_bus_write(uint16_t data, data_width_t width) {

  #if defined(__SAM3X8E__) // If Arduino DUE

    if ((width == EightLow) || (width == Sixteen)) {
      // Set data bus pins to OUTPUT
      PIOB->PIO_OER = BIT26;      // Pin 22
      PIOA->PIO_OER = BIT14 | BIT15; // Pins 23, 24
      PIOD->PIO_OER = BIT00 | BIT01 | BIT02 | BIT03 | BIT06; // Pins 25-29 except 28
    }

    if ((width == EightHigh) || (width == Sixteen)) {
      // Set pins to OUTPUT
      PIOD->PIO_OER = BIT09 | BIT10; // Pins 30 & 32
      PIOA->PIO_OER = BIT07; // Pin 31
      PIOC->PIO_OER = BIT01 | BIT02 | BIT03 | BIT04 | BIT05; // Pins 33-37
    }
    delayMicroseconds(PIN_CHANGE_DELAY);

    if ((width == EightLow) || (width == Sixteen)) {
      // Write low-order byte to data bus pins
      (data & 0x01) ? PIOB->PIO_SODR = BIT26 : PIOB->PIO_CODR = BIT26;      // Pin 22
      (data & 0x02) ? PIOA->PIO_SODR = BIT14 : PIOA->PIO_CODR = BIT14;      // Pin 23
      (data & 0x04) ? PIOA->PIO_SODR = BIT15 : PIOA->PIO_CODR = BIT15;      // Pin 24
      (data & 0x08) ? PIOD->PIO_SODR = BIT00 : PIOD->PIO_CODR = BIT00;      // Pin 25
      (data & 0x10) ? PIOD->PIO_SODR = BIT01 : PIOD->PIO_CODR = BIT01;      // Pin 26
      (data & 0x20) ? PIOD->PIO_SODR = BIT02 : PIOD->PIO_CODR = BIT02;      // Pin 27
      (data & 0x40) ? PIOD->PIO_SODR = BIT03 : PIOD->PIO_CODR = BIT03;      // Pin 28
      (data & 0x80) ? PIOD->PIO_SODR = BIT06 : PIOD->PIO_CODR = BIT06;      // Pin 29
    }

    if ((width == EightHigh) || (width == Sixteen)) {
      (data & 0x0100) ? PIOD->PIO_SODR = BIT09 : PIOD->PIO_CODR = BIT09;    // AD8 Pin 30 (PD9)
      (data & 0x0200) ? PIOA->PIO_SODR = BIT07 : PIOA->PIO_CODR = BIT07;    // AD9 Pin 31 (PA7)
      (data & 0x0400) ? PIOD->PIO_SODR = BIT10 : PIOD->PIO_CODR = BIT10;    // AD10 Pin 32 (PD10)
      (data & 0x0800) ? PIOC->PIO_SODR = BIT01 : PIOC->PIO_CODR = BIT01;    // AD11 Pin 33 (PC1)
      (data & 0x1000) ? PIOC->PIO_SODR = BIT02 : PIOC->PIO_CODR = BIT02;    // AD12 Pin 34 (PC2)
      (data & 0x2000) ? PIOC->PIO_SODR = BIT03 : PIOC->PIO_CODR = BIT03;    // AD13 Pin 35 (PC3)
      (data & 0x4000) ? PIOC->PIO_SODR = BIT04 : PIOC->PIO_CODR = BIT04;    // AD14 Pin 36 (PC4)
      (data & 0x8000) ? PIOC->PIO_SODR = BIT05 : PIOC->PIO_CODR = BIT05;    // AD15 Pin 37 (PC5)
    }
  #elif defined(ARDUINO_GIGA)
    // if (width == EightLow) {
    //   SET_DATA_BUS_TO_WRITE_LO;
    // }
    // else if (width == EightHigh) {
    //   SET_DATA_BUS_TO_WRITE_HI;
    // }
    // else {
    //   SET_DATA_BUS_TO_WRITE;
    // }
    SET_DATA_BUS_TO_WRITE;

    if ((width == EightLow) || (width == Sixteen)) {
      WRITE_BIT(data, 0x01, SET_DBUS_00, CLEAR_DBUS_00);
      WRITE_BIT(data, 0x02, SET_DBUS_01, CLEAR_DBUS_01);
      WRITE_BIT(data, 0x04, SET_DBUS_02, CLEAR_DBUS_02);
      WRITE_BIT(data, 0x08, SET_DBUS_03, CLEAR_DBUS_03);
      WRITE_BIT(data, 0x10, SET_DBUS_04, CLEAR_DBUS_04);
      WRITE_BIT(data, 0x20, SET_DBUS_05, CLEAR_DBUS_05);
      WRITE_BIT(data, 0x40, SET_DBUS_06, CLEAR_DBUS_06);
      WRITE_BIT(data, 0x80, SET_DBUS_07, CLEAR_DBUS_07);
    }

    if ((width == EightHigh) || (width == Sixteen)) {
      WRITE_BIT(data, 0x0100, SET_DBUS_08, CLEAR_DBUS_08);
      WRITE_BIT(data, 0x0200, SET_DBUS_09, CLEAR_DBUS_09);
      WRITE_BIT(data, 0x0400, SET_DBUS_10, CLEAR_DBUS_10);
      WRITE_BIT(data, 0x0800, SET_DBUS_11, CLEAR_DBUS_11);
      WRITE_BIT(data, 0x1000, SET_DBUS_12, CLEAR_DBUS_12);
      WRITE_BIT(data, 0x2000, SET_DBUS_13, CLEAR_DBUS_13);
      WRITE_BIT(data, 0x4000, SET_DBUS_14, CLEAR_DBUS_14);
      WRITE_BIT(data, 0x8000, SET_DBUS_15, CLEAR_DBUS_15);        
    }

    debugPrintlnColor(ansi::green, "## data_bus_write(): done!");

  #elif defined(__AVR_ATmega2560__) // If Arduino MEGA  
    // Set data bus pins 22-29 to OUTPUT
    DDRA = 0xFF;
    delayMicroseconds(PIN_CHANGE_DELAY);
    // TODO: Support 8086
    // Write byte to data bus pins 22-29
    PORTA = data;
  #endif

  CPU.data_bus_resolved = true;
}

// Read a value from the CPU's data bus
uint16_t data_bus_read(data_width_t width) {

  uint16_t data = 0;
  #if defined(__SAM3X8E__) // If Arduino DUE  
    if ((width == EightLow) || (width == Sixteen)) {
      // Set data bus pins to INPUT
      PIOB->PIO_ODR = BIT26;      // Pin 22
      PIOA->PIO_ODR = BIT14 | BIT15; // Pins 23, 24
      PIOD->PIO_ODR = BIT00 | BIT01 | BIT02 | BIT03 | BIT06; // Pins 25-29 except 28
    }
    if ((width == EightHigh) || (width == Sixteen)) {
      // Set pins to INPUT
      PIOD->PIO_ODR = BIT09 | BIT10; // Pins 30 & 32
      PIOA->PIO_ODR = BIT07; // Pin 31
      PIOC->PIO_ODR = BIT01 | BIT02 | BIT03 | BIT04 | BIT05; // Pins 33-37
    }
    delayMicroseconds(PIN_CHANGE_DELAY);

    if ((width == EightLow) || (width == Sixteen)) {
      // Read data from bus pins
      data |= (PIOB->PIO_PDSR & BIT26) ? 0x01 : 0x00;     // Pin 22, Bit 0 of byte
      data |= (PIOA->PIO_PDSR & BIT14) ? 0x02 : 0x00;     // Pin 23, Bit 1 of byte
      data |= (PIOA->PIO_PDSR & BIT15) ? 0x04 : 0x00;     // Pin 24, Bit 2 of byte
      data |= (PIOD->PIO_PDSR & BIT00) ? 0x08 : 0x00;     // Pin 25, Bit 3 of byte
      data |= (PIOD->PIO_PDSR & BIT01) ? 0x10 : 0x00;     // Pin 26, Bit 4 of byte
      data |= (PIOD->PIO_PDSR & BIT02) ? 0x20 : 0x00;     // Pin 27, Bit 5 of byte
      data |= (PIOD->PIO_PDSR & BIT03) ? 0x40 : 0x00;     // Pin 28, Bit 6 of byte
      data |= (PIOD->PIO_PDSR & BIT06) ? 0x80 : 0x00;     // Pin 29, Bit 7 of byte
    }

    if ((width == EightHigh) || (width == Sixteen)) {
      data |= PIOD->PIO_PDSR & BIT09 ? 0x0100 : 0x0000;   // AD8 Pin 30 (PD9)
      data |= PIOA->PIO_PDSR & BIT07 ? 0x0200 : 0x0000;   // AD9 Pin 31 (PA7)
      data |= PIOD->PIO_PDSR & BIT10 ? 0x0400 : 0x0000;   // AD10 Pin 32 (PD10)
      data |= PIOC->PIO_PDSR & BIT01 ? 0x0800 : 0x0000;   // AD11 Pin 33 (PC1)
      data |= PIOC->PIO_PDSR & BIT02 ? 0x1000 : 0x0000;   // AD12 Pin 34 (PC2)
      data |= PIOC->PIO_PDSR & BIT03 ? 0x2000 : 0x0000;   // AD13 Pin 35 (PC3)
      data |= PIOC->PIO_PDSR & BIT04 ? 0x4000 : 0x0000;   // AD14 Pin 36 (PC4)
      data |= PIOC->PIO_PDSR & BIT05 ? 0x8000 : 0x0000;   // AD15 Pin 37 (PC5)
    }
    return data;
  #elif defined(ARDUINO_GIGA)
    SET_DATA_BUS_TO_READ;

    if ((width == EightLow) || (width == Sixteen)) {
      // Read data from bus pins
      if (READ_DBUS_00) data |= 0x0001;
      if (READ_DBUS_01) data |= 0x0002;
      if (READ_DBUS_02) data |= 0x0004;
      if (READ_DBUS_03) data |= 0x0008;
      if (READ_DBUS_04) data |= 0x0010;
      if (READ_DBUS_05) data |= 0x0020;
      if (READ_DBUS_06) data |= 0x0040;
      if (READ_DBUS_07) data |= 0x0080;
    }
    if ((width == EightHigh) || (width == Sixteen)) {
      if (READ_DBUS_08) data |= 0x0100;
      if (READ_DBUS_09) data |= 0x0200;
      if (READ_DBUS_10) data |= 0x0400;
      if (READ_DBUS_11) data |= 0x0800;
      if (READ_DBUS_12) data |= 0x1000;
      if (READ_DBUS_13) data |= 0x2000;
      if (READ_DBUS_14) data |= 0x4000;
      if (READ_DBUS_15) data |= 0x8000;
    }
    return data;
  #elif defined(__AVR_ATmega2560__) // If Arduino MEGA  
    // Set data bus pins 22-29 to INPUT
    DDRA = 0;
    delayMicroseconds(PIN_CHANGE_DELAY);
    // Read LO byte from data bus pins 22-29
    data = PINA;
    
    if (width == Sixteen) {
      // Set data bus pins 30-37 to INPUT
      DDRC = 0;
      delayMicroseconds(PIN_CHANGE_DELAY);
      // Read HO byte from data bus pins 30-37. These are in reversed order for some reason.
      data |= ((uint16_t)reverse_byte(PINC) << 8);
    }
    return data;
  #endif
}

inline uint32_t peek_address() {
  return read_address_pins(true);
}

inline void latch_address() {
  uint32_t addr = read_address_pins(false);
  CPU.address_bus = addr;
  CPU.address_latch = addr;
}

// Return the value of address line 0 as a bool representing if the address is odd
inline bool a0() {
  return (CPU.address_latch & 1) == 1;
}

/*
  Read the address pins and return the 20 bit value in a uint32
  Note: address is only valid while ALE is HIGH (on T1) Otherwise mutiplexed with status and data.
*/
uint32_t read_address_pins(bool peek) {

  uint32_t address = 0;

  #if defined(__SAM3X8E__) // If Arduino DUE  
    
    // If 'peeking' at the bus, we want to see what is being output as well as input. So we don't change
    // pin direction.
    if(!peek) {
      SET_DATA_BUS_TO_READ
    }

    address |= (PIOB->PIO_PDSR & BIT26) ? 0x00000001 : 0;     // AD0  Pin 22 (PB26)
    address |= (PIOA->PIO_PDSR & BIT14) ? 0x00000002 : 0;     // AD1  Pin 23 (PA14)
    address |= (PIOA->PIO_PDSR & BIT15) ? 0x00000004 : 0;     // AD2  Pin 24 (PA15)
    address |= (PIOD->PIO_PDSR & BIT00) ? 0x00000008 : 0;     // AD3  Pin 25 (PD0)
    address |= (PIOD->PIO_PDSR & BIT01) ? 0x00000010 : 0;     // AD4  Pin 26 (PD1)
    address |= (PIOD->PIO_PDSR & BIT02) ? 0x00000020 : 0;     // AD5  Pin 27 (PD2)
    address |= (PIOD->PIO_PDSR & BIT03) ? 0x00000040 : 0;     // AD6  Pin 28 (PD3)
    address |= (PIOD->PIO_PDSR & BIT06) ? 0x00000080 : 0;     // AD7  Pin 29 (PD6)
    address |= (PIOD->PIO_PDSR & BIT09) ? 0x00000100 : 0;     // AD8  Pin 30 (PD9)
    address |= (PIOA->PIO_PDSR & BIT07) ? 0x00000200 : 0;     // AD9  Pin 31 (PA7)
    address |= (PIOD->PIO_PDSR & BIT10) ? 0x00000400 : 0;     // AD10 Pin 32 (PD10)

    address |= (PIOC->PIO_PDSR & BIT01) ? 0x00000800 : 0;     // AD11 Pin 33
    address |= (PIOC->PIO_PDSR & BIT02) ? 0x00001000 : 0;     // AD12 Pin 34
    address |= (PIOC->PIO_PDSR & BIT03) ? 0x00002000 : 0;     // AD13 Pin 35
    address |= (PIOC->PIO_PDSR & BIT04) ? 0x00004000 : 0;     // AD14 Pin 36
    address |= (PIOC->PIO_PDSR & BIT05) ? 0x00008000 : 0;     // AD15 Pin 37
    address |= (PIOC->PIO_PDSR & BIT06) ? 0x00010000 : 0;     // AD16 Pin 38
    address |= (PIOC->PIO_PDSR & BIT07) ? 0x00020000 : 0;     // AD17 Pin 39
    address |= (PIOC->PIO_PDSR & BIT08) ? 0x00040000 : 0;     // AD18 Pin 40
    address |= (PIOC->PIO_PDSR & BIT09) ? 0x00080000 : 0;     // AD19 Pin 41
  #elif defined (ARDUINO_GIGA)
    // If 'peeking' at the bus, we want to see what is being output as well as input. 
    // So we don't change pin direction if peek is true.
    if(!peek) {
      // Set data bus pins to INPUT
      SET_DATA_BUS_TO_READ;
    }
    if (READ_PIN_D22) address |= 0x00000001;  // AD0  Pin 22
    if (READ_PIN_D23) address |= 0x00000002;  // AD1  Pin 23
    if (READ_PIN_D24) address |= 0x00000004;  // AD2  Pin 24
    if (READ_PIN_D25) address |= 0x00000008;  // AD3  Pin 25
    if (READ_PIN_D26) address |= 0x00000010;  // AD4  Pin 26
    if (READ_PIN_D27) address |= 0x00000020;  // AD5  Pin 27
    if (READ_PIN_D28) address |= 0x00000040;  // AD6  Pin 28
    if (READ_PIN_D29) address |= 0x00000080;  // AD7  Pin 29
    if (READ_PIN_D30) address |= 0x00000100;  // AD8  Pin 30
    if (READ_PIN_D31) address |= 0x00000200;  // AD9  Pin 31
    if (READ_PIN_D32) address |= 0x00000400;  // AD10 Pin 32
    if (READ_PIN_D33) address |= 0x00000800;  // AD11 Pin 33
    if (READ_PIN_D34) address |= 0x00001000;  // AD12 Pin 34
    if (READ_PIN_D35) address |= 0x00002000;  // AD13 Pin 35
    if (READ_PIN_D36) address |= 0x00004000;  // AD14 Pin 36
    if (READ_PIN_D37) address |= 0x00008000;  // AD15 Pin 37
    if (READ_PIN_D38) address |= 0x00010000;  // AD16 Pin 38
    if (READ_PIN_D39) address |= 0x00020000;  // AD17 Pin 39
    if (READ_PIN_D40) address |= 0x00040000;  // AD18 Pin 40
    if (READ_PIN_D41) address |= 0x00080000;  // AD19 Pin 41
  #elif defined(__AVR_ATmega2560__) // If Arduino MEGA 
    // Set data bus pins 22-29 to INPUT
    if (!peek) {
      SET_DATA_BUS_TO_READ
    }
    address = PINA; // Pins 22-29
    address |= (unsigned long)BIT_REVERSE_TABLE[PINC] << 8; // Pins 30-37 (Bit order reversed)
    address |= (unsigned long)(PIND & 0x80) << 9; // Pin 38
    address |= (unsigned long)(BIT_REVERSE_TABLE[PING] & 0xE0) << 12; // Pins 39-40 (Bit order reversed)  
  #endif

  return address;
}
