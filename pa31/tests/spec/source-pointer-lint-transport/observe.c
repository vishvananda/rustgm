#include <stdint.h>
#include <stdio.h>

extern uint8_t observed_free(uint8_t value);
extern uint8_t observed_method(uint8_t value);

int main(void) {
    printf("%u %u\n", (unsigned)observed_free(41), (unsigned)observed_method(73));
    return 0;
}
