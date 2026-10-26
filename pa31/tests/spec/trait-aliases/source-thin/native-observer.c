#include <stdint.h>
#include <stdio.h>
extern uint8_t source_thin_result(const uint8_t *);
int main(void) { uint8_t value = 19; printf("%u\n", (unsigned)source_thin_result(&value)); return 0; }
