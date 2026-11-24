#include <stdint.h>
#include <stdio.h>

extern uint8_t observed_first(void);
extern uint8_t observed_second(void);

int main(void) {
    printf("%u %u\n", (unsigned)observed_first(), (unsigned)observed_second());
    return 0;
}
