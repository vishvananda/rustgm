#include <stdint.h>
#include <inttypes.h>
#include <stdio.h>
static volatile uint16_t input = 0x3e00;
uint16_t seed_half(void) { return input; }
extern uint32_t runtime_cast(void);
int main(void) { printf("%08" PRIx32 "\n", runtime_cast()); return 0; }
