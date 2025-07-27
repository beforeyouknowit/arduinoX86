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

/// @brief BusController emulates a bus controller such as the Intel 8288.
class BusController {

  public:
    bool ale;
    bool mrdc;
    bool amwc;
    bool iorc;
    bool mwtc;
    bool aiowc;
    bool iowc;

    BusController() 
      : ale(false), mrdc(false), amwc(false), iorc(false), mwtc(false), aiowc(false), iowc(false) {
        last_status_ = PASV; // Start in passive state
      }

  private:
    BusStatus last_status_; // S0-S2 of previous cycle
    BusStatus status_; // S0-S2 of current cycle
    BusStatus status_latch_;
    TCycle t_cycle_;

  public:
    BusStatus status() const {
      return status_;
    }

    void reset() {
      ale = false;
      mrdc = false;
      amwc = false;
      iorc = false;
      mwtc = false;
      aiowc = false;
      iowc = false;

      last_status_ = PASV;
      status_ = PASV;
      status_latch_ = PASV;
      t_cycle_ = TI;
    }

    void tick(BusStatus new_status) {
      
      last_status_ = status_;
      status_ = new_status;

      // TODO: Handle wait states
      switch (t_cycle_) {
        case TI:
          break;
        case T1:
          ale = false;
          t_cycle_ = T2;
          switch(status_latch_) {
              case IOR:
                iorc = true;
                break;
              case IOW:
                // Set AIOWC line on T3, IOWC is delayed to T3
                aiowc = true;
                break;
              case MEMW:
                // Set AMWC line on T2, MWTC is delayed to T3
                amwc = true;
                break;
              case CODE:
                mrdc = true;
                break;          
              case MEMR:
                mrdc = true;
                break;           
              default:
                break;       
          }
          break;
        case T2:
          t_cycle_ = T3;
            switch(status_latch_) {
              case INTA:
                break;
              case IOW:
                iowc = true;
                break;
              case MEMW:
                mwtc = true;
                break;
              default:
                break;
            }        
          break;
        case T3:
          t_cycle_ = T4;
          iorc = false;
          amwc = false;
          iowc = false;
          mrdc = false;
          aiowc = false;
          mwtc = false;     
          break;
        case TW:
          break;
        case T4:
          t_cycle_ = TI;
          break;
      }

      if (last_status_ == PASV && status_ != PASV) {
        // We started a bus cycle; enter t1 and set ALE
        ale = true;
        t_cycle_ = T1;
        status_latch_ = status_;
      }
    }

};