#include <stdint.h>
#include <stdio.h>

extern uint8_t observed_final(void);
extern uint8_t observed_override(void);

int main(void) {
    printf("%u %u\n", (unsigned)observed_final(), (unsigned)observed_override());
    return 0;
}
