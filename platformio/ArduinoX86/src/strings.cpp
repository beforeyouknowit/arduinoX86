
#include <Arduino.h>
#include "globals.h"

extern const char RESPONSE_CHRS[] = {
  '!', '.'
};

const char VERSION_DAT[] = {
  'a', 'r', 'd', 'x', '8', '6', ' '
};

const size_t VERSION_DAT_LEN = sizeof(VERSION_DAT);

extern const char MACHINE_STATE_CHARS[] = {
  'R', 'I', 'C', 'J', 'L', 'M', '8', 'P', 'E', 'F', 'X', '9', 'S', 'T', 'D'
};

extern const char * const MACHINE_STATE_STRINGS[] = {
  "Reset",
  "CpuId",
  "CpuSetup",
  "JumpVector",
  "Load",
  "LoadDone",
  "EmuEnter",
  "Prefetch",
  "Execute",
  "ExecuteFinalize",
  "ExecuteDone",
  "EmuExit",
  "Store",
  "StoreDone",
  "Done"
};

extern const char * const CMD_STRINGS[] = {
  "NONE",
  "VERSION",
  "RESET",
  "LOAD",
  "CYCLE",
  "READADDRLATCH",
  "READSTATUS",
  "READ8288CMD",
  "READ8288CTRL",
  "READDATABUS",
  "WRITEDATABUS",
  "FINALIZE",
  "BEGINSTORE",
  "STORE",
  "QUEUELEN",
  "QUEUEBYTES",
  "WRITEPIN",
  "READPIN",
  "GETPGMSTATE",
  "GETLASTERR",
  "GETCYCLESTATE",
  "CGETCYCLESTATE",
  "PREFETCHSTORE",
  "READADDRBUS",
  "CPUTYPE",
  "EMULATE8080",
  "PREFETCH",
  "INVALID",
};