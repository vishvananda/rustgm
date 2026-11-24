#include <stdint.h>
#include <stdio.h>

extern uint8_t observed_const(void);
extern uint8_t observed_runtime(uint8_t);

int main(void) {
    printf("%u %u\n", observed_const(), observed_runtime(73));
    return 0;
}
