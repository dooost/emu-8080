#ifndef emu_8080_h
#define emu_8080_h

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>


typedef struct State8080 State8080;

struct State8080 *state8080_evaluating_next(struct State8080 *ptr);

void state8080_free(struct State8080 *ptr);

struct State8080 *state8080_loading_file_into_memory_at(struct State8080 *ptr,
                                                        const char *path,
                                                        uint16_t index);

struct State8080 *state8080_new(void);

#endif /* emu_8080_h */
