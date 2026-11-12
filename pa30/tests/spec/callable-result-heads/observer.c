#include <stdint.h>
#include <stdio.h>
static unsigned trace;
void mark(uint8_t step) { trace = trace * 10 + step; }
extern uint8_t code(void);
int main(void) { uint8_t value = code(); printf("callable-result %u %u\n", (unsigned)value, trace); return value == 23 && trace == 12 ? 0 : 1; }
