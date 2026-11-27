#include <stdint.h>
#include <stdio.h>

extern uint8_t observe(uint8_t);

int main(void) {
    printf("%u %u\n", (unsigned)observe(37), (unsigned)observe(73));
}
