#include <stdint.h>
#include <stdio.h>

extern uint8_t observed_provider(void);
extern uint8_t observed_local(uint8_t);

int main(void) {
    printf("%u %u\n", observed_provider(), observed_local(73));
    return 0;
}
