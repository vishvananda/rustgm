#include <stdint.h>
#include <stdio.h>

extern uint8_t observed(uint8_t value);

int main(void) {
    printf("%u\n", observed(40));
    return 0;
}
