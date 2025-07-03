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





#if defined(HAT_8088_V1)
  #include "hats/Hat8088.h"
  class Hat8088;
  using HatType = Hat8088;
#elif defined(HAT_80186_3V_V1)
  #include "hats/Hat80186.h"
  class Hat80186;
  using HatType = Hat80186;
#elif defined(HAT_286_5V_V1)
  #include "hats/Hat80286.h"
  class Hat80286;
  using HatType = Hat80286;
#elif defined(HAT_386_3V_V1)
  #include "hats/Hat80386.h"
  class Hat80386;
  using HatType = Hat80386;
#else
  #error "You must define a hat type!"
#endif 
