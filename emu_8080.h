#ifndef emu_8080_h
#define emu_8080_h

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct ConditionCodes {
  uint8_t bits;
} ConditionCodes;
#define ConditionCodes_Z (ConditionCodes){ .bits = (uint8_t)1 }
#define ConditionCodes_S (ConditionCodes){ .bits = (uint8_t)2 }
#define ConditionCodes_P (ConditionCodes){ .bits = (uint8_t)4 }
#define ConditionCodes_CY (ConditionCodes){ .bits = (uint8_t)8 }
#define ConditionCodes_AC (ConditionCodes){ .bits = (uint8_t)16 }
#define ConditionCodes_PAD (ConditionCodes){ .bits = (uint8_t)224 }

typedef struct State8080 {
  uint8_t a;
  uint8_t b;
  uint8_t c;
  uint8_t d;
  uint8_t e;
  uint8_t h;
  uint8_t l;
  uint16_t sp;
  uint16_t pc;
  struct ConditionCodes cc;
  bool interrupt_enabled;
} State8080;

struct State8080 *state8080_evaluating_next(struct State8080 *ptr);

void state8080_free(struct State8080 *ptr);

struct State8080 *state8080_new(void);

#endif /* emu_8080_h */
