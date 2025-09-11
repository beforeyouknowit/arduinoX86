#include <Arduino.h>
#include <arduinoX86.h>
#include <globals.h>
#include "opcodes.h"

// LUT of primary opcode to Mnemonic (Or Group name)
extern const uint8_t OPCODE_REFS[] = {
  0, 0, 0, 0, 0, 0, 1, 2, 3, 3, 3, 3, 3, 3, 1, 2, 4, 4, 4, 4, 4, 4, 1, 2, 5, 5, 5, 5, 5, 5, 1, 2,
  6, 6, 6, 6, 6, 6, 7, 8, 9, 9, 9, 9, 9, 9, 10, 11, 12, 12, 12, 12, 12, 12, 13, 14, 15, 15, 15,
  15, 15, 15, 16, 17, 18, 18, 18, 18, 18, 18, 18, 18, 19, 19, 19, 19, 19, 19, 19, 19, 1, 1, 1, 1,
  1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34,
  35, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 105, 105, 105, 105, 36, 36,
  37, 37, 38, 38, 38, 38, 38, 39, 38, 2, 111, 37, 37, 37, 37, 37, 37, 37, 40, 41, 42, 103, 43,
  44, 45, 46, 38, 38, 38, 38, 47, 48, 49, 50, 36, 36, 51, 52, 53, 54, 55, 56, 38, 38, 38, 38, 38,
  38, 38, 38, 38, 38, 38, 38, 38, 38, 38, 38, 57, 57, 57, 57, 58, 59, 38, 38, 60, 60, 60, 60, 61,
  61, 62, 63, 106, 106, 110, 110, 71, 73, 104, 75, 104, 104, 104, 104, 104, 104, 104, 104, 76,
  77, 78, 79, 80, 80, 81, 81, 82, 83, 84, 83, 80, 80, 81, 81, 85, 104, 86, 87, 89, 90, 107, 107,
  97, 98, 99, 100, 101, 102, 108, 109,
};

extern const uint8_t OPCODE_8080_REFS[] = {
  0, 1, 2, 3, 4, 5, 6, 7, 80, 8, 9, 10, 4, 5, 6, 11, 80, 1, 2, 3, 4, 5, 6, 12, 80, 8, 9, 10, 4,
  5, 6, 13, 80, 1, 14, 3, 4, 5, 6, 15, 80, 8, 16, 10, 4, 5, 6, 17, 80, 1, 18, 3, 4, 5, 6, 19, 80,
  8, 20, 10, 4, 5, 6, 21, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
  22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22,
  22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 22, 23, 22, 22, 22, 22, 22, 22, 22, 22, 22, 24, 24,
  24, 24, 24, 24, 24, 24, 25, 25, 25, 25, 25, 25, 25, 25, 26, 26, 26, 26, 26, 26, 26, 26, 27, 27,
  27, 27, 27, 27, 27, 27, 28, 28, 28, 28, 28, 28, 28, 28, 29, 29, 29, 29, 29, 29, 29, 29, 30, 30,
  30, 30, 30, 30, 30, 30, 31, 31, 31, 31, 31, 31, 31, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41,
  42, 80, 43, 44, 45, 39, 46, 33, 47, 48, 49, 37, 50, 39, 51, 80, 52, 53, 54, 80, 55, 39, 56, 33,
  57, 58, 59, 37, 60, 39, 61, 62, 63, 64, 65, 81, 68, 39, 69, 33, 70, 71, 72, 37, 73, 39, 74, 75,
  76, 77, 78, 80, 79, 39,
};

extern const char * const OPCODE_STRS[] = {
  "ADD",
  "PUSH",
  "POP",
  "OR",
  "ADC",
  "SBB",
  "AND",
  "ES",
  "DAA",
  "SUB",
  "CS",
  "DAS",
  "XOR",
  "SS",
  "AAA",
  "CMP",
  "DS",
  "AAS",
  "INC",
  "DEC",
  "JO",
  "JNO",
  "JB",
  "JNB",
  "JZ",
  "JNZ",
  "JBE",
  "JNBE",
  "JS",
  "JNS",
  "JP",
  "JNP",
  "JL",
  "JNL",
  "JLE",
  "JNLE",
  "TEST",
  "XCHG",
  "MOV",
  "LEA",
  "CBW",
  "CWD",
  "CALLF",
  "PUSHF",
  "POPF",
  "SAHF",
  "LAHF",
  "MOVSB",
  "MOVSW",
  "CMPSB",
  "CMPSW",
  "STOSB",
  "STOSW",
  "LODSB",
  "LODSW",
  "SCASB",
  "SCASW",
  "RETN",
  "LES",
  "LDS",
  "RETF",
  "INT",
  "INTO",
  "IRET",
  "ROL",
  "ROR",
  "RCL",
  "RCR",
  "SHL",
  "SHR",
  "SAR",
  "AAM",
  "AMX",
  "AAD",
  "ADX",
  "XLAT",
  "LOOPNE",
  "LOOPE",
  "LOOP",
  "JCXZ",
  "IN",
  "OUT",
  "CALL",
  "JMP",
  "JMPF",
  "LOCK",
  "REPNZ",
  "REP",
  "REPZ",
  "HLT",
  "CMC",
  "NOT",
  "NEG",
  "MUL",
  "IMUL",
  "DIV",
  "IDIV",
  "CLC",
  "STC",
  "CLI",
  "STI",
  "CLD",
  "STD",
  "WAIT",
  "INVAL",
  "GRP1",
  "GRP2A",
  "GRP3",
  "GRP4",
  "GRP5",
  "GRP2B",
  "NOP",
  
};

// 0x80 - 0x81
extern const char * const OPCODE_STRS_GRP1[] = {
  "ADD",
  "OR",
  "ADC",
  "SBB",
  "AND",
  "SUB",
  "XOR",
  "CMP"
};

// 0xD0 - 0xD1
extern const char * const OPCODE_STRS_GRP2A[] = {
  "ROL",
  "ROR",
  "RCL",
  "RCR",
  "SHL",
  "SHR",
  "SETMO",
  "SAR"
};

// 0xD2 - 0xD3
extern const char * const OPCODE_STRS_GRP2B[] = {
  "ROL",
  "ROR",
  "RCL",
  "RCR",
  "SHL",
  "SHR",
  "SETMOC",
  "SAR"
};

// 0xF6 - 0xF7
extern const char * const OPCODE_STRS_GRP3[] = {
  "TEST",
  "TEST",
  "NOT",
  "NEG",
  "MUL",
  "IMUL",
  "DIV",
  "IDIV",
};

// 0xFE
extern const char * const OPCODE_STRS_GRP4[] = {
  "INC",
  "DEC",
  "INVAL",
  "INVAL",
  "INVAL",
  "INVAL",
  "INVAL",
  "INVAL"
};

// 0xFF
extern const char * const OPCODE_STRS_GRP5[] = {
  "INC",
  "DEC",
  "CALL",
  "CALLF",
  "JMP",
  "JMPF",
  "PUSH",
  "INVAL"
};

extern const char * const OPCODE_8080_STRS[] = {
  "NOP",
  "LXI",
  "STAX",
  "INX",
  "INR",
  "DCR",
  "MVI",
  "RLC",
  "DAD",
  "LDAX",
  "DCX",
  "RRC",
  "RAL",
  "RAR",
  "SHLD",
  "DAA",
  "LHLD",
  "CMA",
  "STA",
  "STC",
  "LDA",
  "CMC",
  "MOV",
  "HLT",
  "ADD",
  "ADC",
  "SUB",
  "SBB",
  "ANA",
  "XRA",
  "ORA",
  "CMP",
  "RNZ",
  "POP",
  "JNZ",
  "JMP",
  "CNZ",
  "PUSH",
  "ADI",
  "RST",
  "RZ",
  "RET",
  "JZ",
  "CZ",
  "CALL",
  "ACI",
  "RNC",
  "JNC",
  "OUT",
  "CNC",
  "SUI",
  "RC",
  "JC",
  "IN",
  "CC",
  "SBI",
  "RPO",
  "JPO",
  "XTHL",
  "CPO",
  "ANI",
  "RPE",
  "PCHL",
  "JPE",
  "XCHG",
  "CPE",
  "CALLN",
  "RETEM",
  "XRI",
  "RP",
  "JP",
  "DI",
  "CP",
  "ORI",
  "RM",
  "SPHL",
  "JM",
  "EI",
  "CM",
  "CPI",
  "INVAL",
  "EXT",
};


// ----------------------------------Opcodes-----------------------------------
const char *get_80_opcode_str(uint8_t op1, uint8_t op2) {
  size_t op_idx = (size_t)OPCODE_8080_REFS[op1];

  if (op1 == 0xED) {
    if (op2 == 0xEF) {
      return "CALLN";
    }
    else if (op2 == 0xFD) {
      return "RETEM";
    }
    else {
      return "INVAL";
    }
  }

  return OPCODE_8080_STRS[(size_t)op_idx];
}

// Return the mnemonic name for the specified opcode. If the opcode is a group
// opcode, op2 should be specified and modrm set to true.
const char *get_86_opcode_str(uint8_t op1, uint8_t op2, bool modrm) {

  size_t op_idx = (size_t)OPCODE_REFS[op1];
  size_t grp_idx = 0;

  if(!modrm) {
    // Just return primary opcode
    return OPCODE_STRS[op_idx];
  }
  else {
    // modrm is in use, check if this is a group instruction...
    if(IS_GRP_OP(op1)) {  
      // Lookup opcode group
      grp_idx = MODRM_OP(op2);

      switch(OPCODE_REFS[op1]) {
        case GRP1:
          return OPCODE_STRS_GRP1[grp_idx];
          break;        
        case GRP2A:
          return OPCODE_STRS_GRP2A[grp_idx];        
          break;    
        case GRP2B:
          return OPCODE_STRS_GRP2B[grp_idx];         
          break;                   
        case GRP3:
          return OPCODE_STRS_GRP3[grp_idx];        
          break;        
        case GRP4:
          return OPCODE_STRS_GRP4[grp_idx];          
          break;        
        case GRP5:
          return OPCODE_STRS_GRP5[grp_idx];         
          break;
        default:
          return "***";
          break;
      }
    }
    else {
      // Not a group instruction, just return as normal
      return OPCODE_STRS[op_idx];
    }
  }
}

const char *get_opcode_str(uint8_t op1, uint8_t op2, bool modrm) {
  if (CPU.in_emulation) {
    return get_80_opcode_str(op1, op2);
  }
  else {
    return get_86_opcode_str(op1, op2, modrm);
  }
}


