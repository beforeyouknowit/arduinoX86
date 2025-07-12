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
#include <stdint.h>
#include <string.h> // for memset
#include <Arduino.h>

#ifdef ARDUINO_GIGA
  #include <SDRAM.h>
  #define HT_ALLOC(sz) SDRAM.malloc(sz)
  #define HT_FREE(ptr) SDRAM.free(ptr)
#else
  #include <cstdlib>
  #define HT_ALLOC(sz) malloc(sz)
  #define HT_FREE(ptr) free(ptr)
#endif

class StaticHashTable {
public:
  struct Entry {
    uint32_t key;
    uint16_t value;
    bool in_use;
  };

  StaticHashTable(size_t capacity)
      : capacity_(capacity), count_(0)
  {
    // Capacity must be power of two
    shift_ = 32 - __builtin_ctz(capacity_);
    entry_pool_ = static_cast<Entry *>(HT_ALLOC(sizeof(Entry) * capacity_));
    if (entry_pool_) {
      memset(entry_pool_, 0, sizeof(Entry) * capacity_);
    }
  }

  ~StaticHashTable() {
    if (entry_pool_) {
      HT_FREE(entry_pool_);
    }
  }

  bool insert(uint32_t key, uint16_t value) {
    if (!entry_pool_) return false;

    size_t index = hash(key);
    for (size_t i = 0; i < capacity_; ++i) {
      Entry &entry = entry_pool_[index];
      if (!entry.in_use || entry.key == key) {
        entry.key = key;
        entry.value = value;
        if (!entry.in_use) {
          entry.in_use = true;
          ++count_;
        }
        return true;
      }
      index = (index + 1) & (capacity_ - 1);
    }
    return false;
  }

  bool find(uint32_t key, uint16_t &value_out) const {
    if (!entry_pool_) return false;

    size_t index = hash(key);
    for (size_t i = 0; i < capacity_; ++i) {
      const Entry &entry = entry_pool_[index];
      if (!entry.in_use) return false;
      if (entry.key == key) {
        value_out = entry.value;
        return true;
      }
      index = (index + 1) & (capacity_ - 1);
    }
    return false;
  }

  void clear() {
    if (entry_pool_) {
      memset(entry_pool_, 0, sizeof(Entry) * capacity_);
      count_ = 0;
    }
  }

  size_t size() const { return count_; }
  size_t capacity() const { return capacity_; }

private:
  size_t hash(uint32_t key) const {
    return (key * 2654435761u) >> shift_;
  }

  Entry *entry_pool_ = nullptr;
  size_t capacity_;
  size_t count_;
  uint8_t shift_;
};