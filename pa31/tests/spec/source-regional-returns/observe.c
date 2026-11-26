#include <stdint.h>
#include <stdio.h>

extern uint8_t good(uint8_t value);
extern uint8_t failed(uint8_t code);

int main(void) {
    printf("%u %u %u %u\n", good(25), good(18), failed(35), failed(76));
    return 0;
}
