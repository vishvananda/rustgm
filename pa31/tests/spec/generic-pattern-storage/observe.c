#include <stdint.h>
#include <stdio.h>
extern uint8_t entry(void);
extern uint8_t probe(uint8_t value);
int main(void) { printf("%u %u\n", entry(), probe(59)); }
