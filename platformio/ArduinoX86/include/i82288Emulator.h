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

#pragma once

#include <BusTypes.h>

/// @brief BusController emulates the Intel 82288 bus controller.
class BusController {

  public:
    BusController()
      : ale_(false), mrdc_(true), iorc_(true), mwtc_(true), iowc_(true), inta_(true) {
        last_status_ = 0x03; // Start in passive state
      }

  private:
    uint8_t last_status_; // S0-S1, M/IO and COD/INTA of previous cycle
    uint8_t status_; // S0-S1, M/IO and COD/INTA of current cycle
    uint8_t status_latch_;
    TCycle t_cycle_;

    bool ale_;
    bool mrdc_;
    bool iorc_;
    bool mwtc_;
    bool iowc_;
    bool inta_;

  public:
    uint8_t status() const {
      return status_;
    }

    void reset() {
      ale_ = false;
      mrdc_ = true;
      iorc_ = true;
      mwtc_ = true;
      iowc_ = true;
      inta_ = true;
      last_status_ = 0x03;
      status_ = 0x03;
      status_latch_ = 0x03;
      t_cycle_ = TI;
    }

    void tick(uint8_t new_status, bool ready) {

      last_status_ = status_;
      status_ = new_status;

      switch (t_cycle_) {
        case TI:
          break;
        case T1:
          ale_ = false;
          t_cycle_ = T2;
          switch(status_latch_ & 0x0F) {
            case 0b0000:
              inta_ = false;
              break;
            case 0b1001:
              iorc_ = false;
              break;
            case 0b1010:
              iowc_ = false;
              break;
            case 0b1101: // FALLTHRU (CODE)
            case 0b0101:
              mrdc_ = false;
              break;
            case 0b0110:
              mwtc_ = false;
              break;          
            case 0b0111: // FALLTHRU (PASV)
            case 0b1111:
              iorc_ = true;
              iowc_ = true;
              mrdc_ = true;
              mwtc_ = true;  
              inta_ = true;
              break;
            default:
              break;       
          }
          break;
        case T2:
          if(ready) {
            // If READY is high we can complete the bus cycle.
            iorc_ = true;
            iowc_ = true;
            mrdc_ = true;
            mwtc_ = true;  
            inta_ = true;
            t_cycle_ = TI;
          }
          break;
        default:
          break;
      }

      if ((last_status_ & 0x03) == 0x03 && (status_ & 0x03) != 0x03) {
        // We started a bus cycle; enter T1 and set ALE
        ale_ = true;
        t_cycle_ = T1;
        status_latch_ = status_;
      }
    }

    bool mrdc() {
      return mrdc_;
    }
    bool iorc() {
      return iorc_;
    }
    bool mwtc() {
      return mwtc_;
    }
    bool iowc() {
      return iowc_;
    }
    bool inta() {
      return inta_;
    }

};