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

#include <Arduino.h>
#include <BusTypes.h>

enum class QueueDataType : uint8_t {
  Program = 0,
  ProgramEnd = 1,
};

class InstructionQueue {

  private:
    size_t size_ = 6; // Capacity of the queue
    size_t len_ = 0; // Number of items in the queue
    size_t back_ = 0; // Index of the back of the queue
    size_t front_ = 0; // Index of the front of the queue
    BusWidth fetch_width_ = BusWidth::Eight; // Data bus width

    uint8_t queue_[6] = {0};
    QueueDataType types_[6] = {QueueDataType::Program}; // Types of data in the queue

  void init_queue() {
    len_ = 0;
    back_ = 0;
    front_ = 0;
  }
  public:
    InstructionQueue(size_t queue_size = 4, BusWidth bus_width = BusWidth::Eight) : size_(queue_size), fetch_width_(bus_width) {
      init_queue();  // Changed from init() to init_queue()
    }

    /// @brief  Return the number of bytes in the queue.
    size_t len() {
      return len_;
    }
    
    /// @brief Return the current capacity of the queue.
    size_t size() {
      return size_;
    }

    /// @brief Push data into the instruction queue.
    /// @param data The data to push into the queue, either 8 or 16 bits.
    /// @param d_type The data type tag for the data being pushed, indicating whether it's a program byte or an end of program byte.
    /// @param width The width of the data being pushed, which can be 8-bit low, 8-bit high, or 16-bit.
    void push(uint16_t data, QueueDataType d_type, ActiveBusWidth width) {
      if (width == ActiveBusWidth::EightLow) {
        // 8-bit low byte fetch (8088/V20)
        if(have_room(width)) {
          queue_[front_] = (uint8_t)data;
          types_[front_] = d_type;
          front_ = (front_ + 1) % size_;
          len_++;
        }
      }
      else if (width == ActiveBusWidth::EightHigh) {
        // 8-bit high byte fetch (8086/V30 odd address)
        if(have_room(width)) {
          queue_[front_] = (uint8_t)(data >> 8);
          types_[front_] = d_type;
          front_ = (front_ + 1) % size_;
          len_++;
        }
      }
      else {
        // 16-bit fetch
        if(have_room(width)) {
          queue_[front_] = (uint8_t)data;
          types_[front_] = d_type;
          front_ = (front_ + 1) % size_;
          queue_[front_] = (uint8_t)(data >> 8);
          types_[front_] = d_type;
          front_ = (front_ + 1) % size_;
          len_ += 2;
        }
      }
    }

  /// @brief Pop a byte and its data type from the queue.
  /// @param byte Pointer to store the popped byte.
  /// @param dtype Pointer to store the data type of the popped byte.
  /// @return True if a byte was popped, false if the queue was empty.
  bool pop(uint8_t *byte, QueueDataType *d_type) {
    if(len_ > 0) {
      *byte = queue_[back_];
      *d_type = types_[back_];
      back_ = (back_ + 1) % size_;
      len_--;
      return true;
    }
    
    return false;
  }

  /// @brief Return true if we have room in the queue for a push
  bool have_room(ActiveBusWidth width) {
    if((width == ActiveBusWidth::EightLow) || (width == ActiveBusWidth::EightHigh)) {
      return (size_t)(len_ + 1) <= size_;
    }
    else {
      return (size_t)(len_ + 2) <= size_;
    }
  }
  /// @brief Flush the queue, returning the number of bytes flushed.
  size_t flush() {
    size_t bytes_flushed = len_;
    init_queue();  // Changed from init() to init_queue()
    return bytes_flushed;
  }

  uint8_t read_byte(size_t idx) {
    if(idx < len_) {
      return queue_[(back_ + idx) % size_];
    }
    else {
      return 0;
    }
  }

  const char *to_string() {
    constexpr size_t buf_len = (6 * 2) + 1;
    static char buf[buf_len];
    char *buf_p = buf;
    *buf_p = 0;
    uint8_t byte;
    for(uint8_t i = 0; i < len_; i++ ) {
      byte = queue_[(back_ + i) % size_];
      snprintf(buf_p, buf_len - (i * 2), "%02X", byte);
      buf_p += 2;
    }  
    return buf;
  }
};

