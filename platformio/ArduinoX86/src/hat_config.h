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

#ifndef _HAT_CONFIG
#define _HAT_CONFIG

#include "arduinoX86.h"

#if defined(__SAM3X8E__) // If Arduino DUE
  #define BOARD_TYPE ARDUINO_DUE
  #define INBAND_SERIAL SerialUSB
  #define DEBUG_SERIAL Serial1
  #define FLUSH SERIAL.flush()
#elif defined(ARDUINO_GIGA) // If Arduino GIGA
  #define BOARD_TYPE ARDUINO_GIGA
  #define INBAND_SERIAL Serial
  #define DEBUG_SERIAL Serial2
  #define FLUSH 
#endif

// This header defines hat-specific GPIO mappings.
// Different ArduinoX86 boards use different GPIO layouts.

// The main AruduinoX86 boards are:
// Arduino8088 rev 1.1 - Designed for the Arduino DUE.
// Arduino286 rev 2 - Designed for the Arduino DUE.
// Arduino286 rev 3 - Designed for the Arduino GIGA.

// Only define one of these!
#define HAT_8088_V1 1
#define HAT_286_V1 0
#define HAT_286_V2 0

// If you stuck an FPU hat on your 8088 HAT define that here
#define HAT_8087_V1 0

#ifdef HAT_8088_V1 
  // Data bus mappings for Arduino8088 hat.
  // 
  #define SET_DBUS_00 do { SET_PIN_D22; } while (0)
  #define CLEAR_DBUS_00 do { CLEAR_PIN_D22; } while (0)

  #define SET_DBUS_01 do { SET_PIN_D23; } while (0)
  #define CLEAR_DBUS_01 do { CLEAR_PIN_D23; } while (0)

  #define SET_DBUS_02 do { SET_PIN_D24; } while (0)
  #define CLEAR_DBUS_02 do { CLEAR_PIN_D24; } while (0)

  #define SET_DBUS_03 do { SET_PIN_D25; } while (0)
  #define CLEAR_DBUS_03 do { CLEAR_PIN_D25; } while (0)

  #define SET_DBUS_04 do { SET_PIN_D26; } while (0)
  #define CLEAR_DBUS_04 do { CLEAR_PIN_D26; } while (0)

  #define SET_DBUS_05 do { SET_PIN_D27; } while (0)
  #define CLEAR_DBUS_05 do { CLEAR_PIN_D27; } while (0)

  #define SET_DBUS_06 do { SET_PIN_D28; } while (0)
  #define CLEAR_DBUS_06 do { CLEAR_PIN_D28; } while (0)

  #define SET_DBUS_07 do { SET_PIN_D29; } while (0)
  #define CLEAR_DBUS_07 do { CLEAR_PIN_D29; } while (0)

  #define SET_DBUS_08 do { SET_PIN_D30; } while (0)
  #define CLEAR_DBUS_08 do { CLEAR_PIN_D30; } while (0)

  #define SET_DBUS_09 do { SET_PIN_D31; } while (0)
  #define CLEAR_DBUS_09 do { CLEAR_PIN_D31; } while (0)

  #define SET_DBUS_10 do { SET_PIN_D32; } while (0)
  #define CLEAR_DBUS_10 do { CLEAR_PIN_D32; } while (0)

  #define SET_DBUS_11 do { SET_PIN_D33; } while (0)
  #define CLEAR_DBUS_11 do { CLEAR_PIN_D33; } while (0)

  #define SET_DBUS_12 do { SET_PIN_D34; } while (0)
  #define CLEAR_DBUS_12 do { CLEAR_PIN_D34; } while (0)

  #define SET_DBUS_13 do { SET_PIN_D35; } while (0)
  #define CLEAR_DBUS_13 do { CLEAR_PIN_D35; } while (0)

  #define SET_DBUS_14 do { SET_PIN_D36; } while (0)
  #define CLEAR_DBUS_14 do { CLEAR_PIN_D36; } while (0)

  #define SET_DBUS_15 do { SET_PIN_D37; } while (0)
  #define CLEAR_DBUS_15 do { CLEAR_PIN_D37; } while (0)

  #define READ_DBUS_00 READ_PIN_D22
  #define READ_DBUS_01 READ_PIN_D23
  #define READ_DBUS_02 READ_PIN_D24
  #define READ_DBUS_03 READ_PIN_D25
  #define READ_DBUS_04 READ_PIN_D26
  #define READ_DBUS_05 READ_PIN_D27
  #define READ_DBUS_06 READ_PIN_D28
  #define READ_DBUS_07 READ_PIN_D29
  #define READ_DBUS_08 READ_PIN_D30
  #define READ_DBUS_09 READ_PIN_D31
  #define READ_DBUS_10 READ_PIN_D32
  #define READ_DBUS_11 READ_PIN_D33
  #define READ_DBUS_12 READ_PIN_D34
  #define READ_DBUS_13 READ_PIN_D35
  #define READ_DBUS_14 READ_PIN_D36
  #define READ_DBUS_15 READ_PIN_D37

  #define READ_ABUS_00 READ_PIN_D22
  #define READ_ABUS_01 READ_PIN_D23
  #define READ_ABUS_02 READ_PIN_D24
  #define READ_ABUS_03 READ_PIN_D25
  #define READ_ABUS_04 READ_PIN_D26
  #define READ_ABUS_05 READ_PIN_D27
  #define READ_ABUS_06 READ_PIN_D28
  #define READ_ABUS_07 READ_PIN_D29
  #define READ_ABUS_08 READ_PIN_D30
  #define READ_ABUS_09 READ_PIN_D31
  #define READ_ABUS_10 READ_PIN_D32
  #define READ_ABUS_11 READ_PIN_D33
  #define READ_ABUS_12 READ_PIN_D34
  #define READ_ABUS_13 READ_PIN_D35
  #define READ_ABUS_14 READ_PIN_D36
  #define READ_ABUS_15 READ_PIN_D37
  #define READ_ABUS_16 READ_PIN_D38
  #define READ_ABUS_17 READ_PIN_D39
  #define READ_ABUS_18 READ_PIN_D40
  #define READ_ABUS_19 READ_PIN_D41

  #if defined(__SAM3X8E__) // If Arduino DUE
    #define SET_DATA_BUS_TO_READ do { \
      uint32_t pins_b = BIT26; \
      uint32_t pins_a = BIT07 | BIT14 | BIT15; \
      uint32_t pins_c = 0x01FF; \
      uint32_t pins_d = BIT00 | BIT01 | BIT02 | BIT03 | BIT06 | BIT09 | BIT10; \
      PIOA->PIO_ODR = pins_a; \
      PIOB->PIO_ODR = pins_b; \
      PIOC->PIO_ODR = pins_c; \
      PIOD->PIO_ODR = pins_d; \
      delayMicroseconds(PIN_CHANGE_DELAY); \
    } while (0)
  #elif defined(ARDUINO_GIGA) // If Arduino GIGA
    #define PORT_K_DBUS_MASK 0xFFFFC03F
    #define PORT_K_DBUS_READ 0
    #define PORT_K_DBUS_WRITE 0x00001540
    #define PORT_J_DBUS_MASK 0x0CFFC000
    #define PORT_J_DBUS_READ 0
    #define PORT_J_DBUS_WRITE 0x51411555
    #define PORT_J_DBUS_WRITE_HI 0x04101540
    #define PORT_J_DBUS_WRITE_LO 0x51410015 
    #define PORT_G_DBUS_MASK 0xF0FFFFFF
    #define PORT_G_DBUS_READ 0
    #define PORT_G_DBUS_WRITE 0x05000000

    #define SET_DATA_BUS_TO_READ do { \
      GPIOK->MODER = (GPIOK->MODER & PORT_K_DBUS_MASK) | PORT_K_DBUS_READ; \
      GPIOJ->MODER = (GPIOJ->MODER & PORT_J_DBUS_MASK) | PORT_J_DBUS_READ; \
      GPIOG->MODER = (GPIOG->MODER & PORT_G_DBUS_MASK) | PORT_G_DBUS_READ; \
      PIN_CHANGE_DELAY; \
    } while (0)

    #define SET_DATA_BUS_TO_WRITE_LO do { \
      GPIOJ->MODER = (GPIOJ->MODER & PORT_J_DBUS_MASK) | PORT_J_DBUS_WRITE_LO; \
      GPIOG->MODER = (GPIOG->MODER & PORT_G_DBUS_MASK) | PORT_G_DBUS_WRITE; \
      PIN_CHANGE_DELAY; \
    } while (0)

    #define SET_DATA_BUS_TO_WRITE_HI do { \
      GPIOK->MODER = (GPIOK->MODER & PORT_K_DBUS_MASK) | PORT_K_DBUS_WRITE; \
      GPIOJ->MODER = (GPIOJ->MODER & PORT_J_DBUS_MASK) | PORT_J_DBUS_WRITE_HI; \
      PIN_CHANGE_DELAY; \
    } while (0)

    #define SET_DATA_BUS_TO_WRITE do { \
      GPIOK->MODER = (GPIOK->MODER & PORT_K_DBUS_MASK) | PORT_K_DBUS_WRITE; \
      GPIOJ->MODER = (GPIOJ->MODER & PORT_J_DBUS_MASK) | PORT_J_DBUS_WRITE; \
      GPIOG->MODER = (GPIOG->MODER & PORT_G_DBUS_MASK) | PORT_G_DBUS_WRITE; \
      PIN_CHANGE_DELAY; \
    } while (0)
  #endif
#endif

// -----------------------------Buzzer ----------------------------------------
#define BUZZER_PIN 2

// ------------------------- CPU Control pins ---------------------------------

#define CLK_PIN 4
#define RESET_PIN 5

#if defined(__AVR_ATmega2560__) // If Arduino MEGA
  
  #define WRITE_BUZZER(x) ((x) ? (PORTE |= (1 << 4)) : (PORTE &= ~(1 << 4)))

#elif defined(__SAM3X8E__) // If Arduino DUE

  #define WRITE_BUZZER(x) ((x) ? (PIOB->PIO_SODR = PIO_PB25) : (PIOB->PIO_CODR = PIO_PB25))

#elif defined(ARDUINO_GIGA)

  // do buzzer here
  #define WRITE_BUZZER(x) ((X))
#endif

// -------------------------- CPU Input pins ----------------------------------
#define BHE_PIN 17
#define READ_BHE_PIN READ_PIN_D17
#define READ_READY_PIN READ_PIN_D06
#define READ_TEST_PIN READ_PIN_D07
#define READ_INTR_PIN READ_PIN_D12
#define READ_NMI_PIN READ_PIN_D13
#define READ_S0_PIN READ_PIN_D14
#define READ_S1_PIN READ_PIN_D15
#define READ_S2_PIN READ_PIN_D16
#define READ_S3_PIN READ_PIN_D38
#define READ_S4_PIN READ_PIN_D39
#define READ_S5_PIN READ_PIN_D40
#define READ_QS0_PIN READ_PIN_D09
#define READ_QS1_PIN READ_PIN_D08

#define READY_PIN 6
#define TEST_PIN 7
#define LOCK_PIN 10
#define INTR_PIN 12
#define NMI_PIN 13

// -------------------------- CPU Output pins ---------------------------------
#define RQ_PIN 3

// --------------------------8288 Control Inputs ------------------------------
#define AEN_PIN 54
#define CEN_PIN 55

// --------------------------8288 Control lines -------------------------------
#define ALE_PIN 50
#define DTR_PIN 49
#define MCEPDEN_PIN 43
#define DEN_PIN 44

// --------------------------8288 Command lines -------------------------------
#define MRDC_PIN 51
#define AMWC_PIN 52
#define MWTC_PIN 53
#define IORC_PIN 46
#define AIOWC_PIN 48
#define IOWC_PIN 47
#define INTA_PIN 45

// -------------------------- Macro definitions  ---------------------------------

// Write macros
#if defined(__SAM3X8E__) // If Arduino DUE
  // D4: PC26* (some references say PA29 - didn't work)
  #define WRITE_CLK(x) ((x) ? (PIOC->PIO_SODR = BIT26) : (PIOC->PIO_CODR = BIT26))
  // D5: PC25
  #define WRITE_RESET(x) ((x) ? (PIOC->PIO_SODR = PIO_PC25) : (PIOC->PIO_CODR = PIO_PC25))
  // D6: PC24
  #define WRITE_READY_PIN(x) ((x) ? (PIOC->PIO_SODR = BIT24) : (PIOC->PIO_CODR = BIT24))
  // D7: PC23
  #define WRITE_TEST_PIN(x) ((x) ? (PIOC->PIO_SODR = BIT23) : (PIOC->PIO_CODR = BIT23))
  // D10: PC29*
  #define WRITE_LOCK_PIN(x) ((x) ? (PIOC->PIO_SODR = BIT29) : (PIOC->PIO_CODR = BIT29))
  // D12: PD8
  #define WRITE_INTR_PIN(x) ((x) ? (PIOD->PIO_SODR = BIT08) : (PIOD->PIO_CODR = BIT08))
  // D13: PB27
  #define WRITE_NMI_PIN(x) ((x) ? (PIOB->PIO_SODR = BIT27) : (PIOB->PIO_CODR = BIT27))
  // A0: PA16
  #define WRITE_AEN_PIN(x) ((x) ? (PIOA->PIO_SODR = BIT16) : (PIOA->PIO_CODR = BIT16))
  // A1: PA24
  #define WRITE_CEN_PIN(x) ((x) ? (PIOA->PIO_SODR = BIT24) : (PIOA->PIO_CODR = BIT24))

#elif defined(__AVR_ATmega2560__) // If Arduino MEGA
  // D4
  #define WRITE_CLK(x) ((x) ? (PORTG |= (1 << 5)) : (PORTG &= ~(1 << 5))) // CLK is PG5
  // D5
  #define WRITE_RESET(x) ((x) ? (PORTE |= (1 << 3)) : (PORTE &= ~(1 << 3))) // RESET is PE3
  // D6
  #define WRITE_READY_PIN(x) ((x) ? (PORTH |= (1 << 3)) : (PORTH &= ~(1 << 3)))
  // D7
  #define WRITE_TEST_PIN(x) ((x) ? (PORTH |= (1 << 4)) : (PORTH &= ~(1 << 4)))
  // D10
  #define WRITE_LOCK_PIN(x) ((x) ? (PORTB |= (1 << 4)) : (PORTB &= ~(1 << 4)))
  // D12
  #define WRITE_INTR_PIN(x) ((x) ? (PORTB |= (1 << 6)) : (PORTB &= ~(1 << 6)))
  // D13
  #define WRITE_NMI_PIN(x) ((x) ? (PORTB |= (1 << 7)) : (PORTB &= ~(1 << 7)))
  // A0
  #define WRITE_AEN_PIN(x) ((x) ? (PORTF |= 0x01) : (PORTF &= ~0x01))
  // A1
  #define WRITE_CEN_PIN(x) ((x) ? (PORTF |= (1 << 1)) : (PORTF &= ~(1 << 1)))

#elif defined (ARDUINO_GIGA)

  // D4: PJ8
  #define WRITE_CLK(x) WRITE_PIN_D04(x)
  // D5: PA7
  #define WRITE_RESET(x) WRITE_PIN_D05(x)
  // D6: PD13
  #define WRITE_READY_PIN(x) WRITE_PIN_D06(x)
  // D7: PB4
  #define WRITE_TEST_PIN(x) WRITE_PIN_D07(x)
  // D10: PK1
  #define WRITE_LOCK_PIN(x) WRITE_PIN_D10(x)
  // D12: PJ11
  #define WRITE_INTR_PIN(x) WRITE_PIN_D12(x)
  // D13: PH6
  #define WRITE_NMI_PIN(x) WRITE_PIN_D13(x)
  // A0: PC4
  #define WRITE_AEN_PIN(x) WRITE_PIN_A0(x)
  // A1: PC5
  #define WRITE_CEN_PIN(x) WRITE_PIN_A1(x)

#endif 

// Read macros

#if defined(__SAM3X8E__) // If Arduino DUE
  #define READ_LOCK_PIN READ_PIN_D10
#elif defined(__AVR_ATmega2560__) // If Arduino MEGA
  #define READ_LOCK_PIN 0
#elif defined(ARDUINO_GIGA)
  #define READ_LOCK_PIN READ_PIN_D10
#endif


#if EMULATE_8288
  // D50: PC13
  #if CPU_186
    // The 186 has its own ALE pin, so we will defer to that
    #define READ_ALE_PIN  READ_PIN_D50
  #else
    #define READ_ALE_PIN  (I8288.ale)
  #endif
  // D51: PC12
  #define READ_MRDC_PIN   (!I8288.mrdc)
  // D52: PB21
  #define READ_AMWC_PIN   (!I8288.amwc)
  // D53: PB14
  #define READ_MWTC_PIN   (!I8288.mwtc)
  // D46: PC17
  #define READ_IORC_PIN   (!I8288.iorc)
  // D48: PC15
  #define READ_AIOWC_PIN  (!I8288.aiowc)
  // D47: PC16
  #define READ_IOWC_PIN   (!I8288.iowc)
  // D45: PC18
  #define READ_INTA_PIN   (!I8288.inta)
#else
  #if defined(__SAM3X8E__) // If Arduino DUE
    
    #if CPU_186
      // The L186 doesn't use an 8288 and can produce its own bus signals, but they need to be 
      // decoded 
      #define READ_ALE_PIN      READ_PIN_D50
      #define READ_MRDC_PIN     !(!READ_PIN_D51 && READ_PIN_D16)    // We hook !RD up to D51. Mem read when S2 (D16) is high.
      #define READ_AMWC_PIN     1                                   // There is no AMWC signal. Simulate inactive-high.
      #define READ_MWTC_PIN     !(!READ_PIN_D53 && READ_PIN_D16)    // We hook !WR up to D53. Mem write when S2 (D16) is high.
      #define READ_IORC_PIN     !(!READ_PIN_D51 && !READ_PIN_D16)   // We hook !RD up to D51. IO read when S2 (D16) is low.
      #define READ_AIOWC_PIN    1                                   // There is no AIOWC signal. Simulate inactive-high.
      #define READ_IOWC_PIN     !(!READ_PIN_D53 && !READ_PIN_D16)   // We hook !WR up to D53. IO write when S2 (D16) is low.
      #define READ_INTA_PIN     READ_PIN_D45
    #else
      #define READ_AEN_PIN      ((PIOD->PIO_PDSR & BIT10) != 0)
      #define READ_CEN_PIN      ((PIOD->PIO_PDSR & BIT09) != 0)

      // D50: PC13
      #define READ_ALE_PIN      READ_PIN_D50
      #define READ_DTR_PIN      ((PIOC->PIO_PDSR & BIT03) != 0)
      #define READ_MCEPDEN_PIN  ((PIOC->PIO_PDSR & BIT01) != 0)
      #define READ_DEN_PIN      ((PIOC->PIO_PDSR & BIT02) != 0)

      #define READ_MRDC_PIN     READ_PIN_D51
      #define READ_AMWC_PIN     READ_PIN_D52
      #define READ_MWTC_PIN     READ_PIN_D53
      #define READ_IORC_PIN     READ_PIN_D46
      #define READ_AIOWC_PIN    READ_PIN_D48
      #define READ_IOWC_PIN     READ_PIN_D47
      #define READ_INTA_PIN     READ_PIN_D45
    #endif
  #elif defined(ARDUINO_GIGA)
    #if CPU_186
      // The L186 doesn't use an 8288 and can produce its own bus signals, but they need to be 
      // decoded 
      #define READ_ALE_PIN      READ_PIN_D50
      #define READ_MRDC_PIN     !(!READ_PIN_D51 && READ_PIN_D16)    // We hook !RD up to D51. Mem read when S2 (D16) is high.
      #define READ_AMWC_PIN     1                                   // There is no AMWC signal. Simulate inactive-high.
      #define READ_MWTC_PIN     !(!READ_PIN_D53 && READ_PIN_D16)    // We hook !WR up to D53. Mem write when S2 (D16) is high.
      #define READ_IORC_PIN     !(!READ_PIN_D51 && !READ_PIN_D16)   // We hook !RD up to D51. IO read when S2 (D16) is low.
      #define READ_AIOWC_PIN    1                                   // There is no AIOWC signal. Simulate inactive-high.
      #define READ_IOWC_PIN     !(!READ_PIN_D53 && !READ_PIN_D16)   // We hook !WR up to D53. IO write when S2 (D16) is low.
      #define READ_INTA_PIN     READ_PIN_D45
    #else
      #define READ_ALE_PIN      READ_PIN_D50
      #define READ_MRDC_PIN     READ_PIN_D51
      #define READ_AMWC_PIN     READ_PIN_D52
      #define READ_MWTC_PIN     READ_PIN_D53
      #define READ_IORC_PIN     READ_PIN_D46
      #define READ_AIOWC_PIN    READ_PIN_D48
      #define READ_IOWC_PIN     READ_PIN_D47
      #define READ_INTA_PIN     READ_PIN_D45
    #endif    
  #elif defined(__AVR_ATmega2560__) // If Arduino MEGA

    // TODO: implement me
    #define READ_LOCK_PIN 0

    #define READ_AEN_PIN ((PINF & 0x01) != 0)
    #define READ_CEN_PIN ((PINF & 0x02) != 0)

    #define READ_ALE_PIN ((PINB & 0x08) != 0)
    #define READ_DTR_PIN ((PINL & 0x01) != 0)
    #define READ_MCEPDEN_PIN ((PINL & 0x40) != 0) 
    #define READ_DEN_PIN ((PINL & 0x20) != 0)

    #define READ_MRDC_PIN ((PINB & 0x04) != 0)
    #define READ_AMWC_PIN ((PINB & 0x02) != 0)
    #define READ_MWTC_PIN ((PINB & 0x01) != 0)
    #define READ_IORC_PIN ((PINL & 0x08) != 0)
    #define READ_AIOWC_PIN ((PINL & 0x02) != 0)
    #define READ_IOWC_PIN ((PINL & 0x04) != 0)
    #define READ_INTA_PIN ((PINL & 0x10) != 0)
  #endif
#endif

// Address pins, used for slow address reading via digitalRead()
const int ADDRESS_PINS[] = {
  22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41
};
const int ADDRESS_LINES = 20;

// All output pins, used to set pin direction on setup
const int OUTPUT_PINS[] = {
  4,  // CLK
  5,  // RESET
  6,  // READY
  7,  // TEST
  12, // INTR
  13, // NMI,
  54, // AEN,
  55, // CEN
};

// All input pins, used to set pin direction on setup
const int INPUT_PINS[] = {
  3,8,9,10,11,14,15,16,17,
  22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,
  43,44,45,46,47,48,49,50,51,52,53
};

#endif // _HAT_CONFIG