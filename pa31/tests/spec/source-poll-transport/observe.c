#include <stdint.h>
#include <stdio.h>

extern uint8_t observed(uint8_t);

int main(void) {
    printf("%u %u %u\n", observed(0), observed(1), observed(2));
    return 0;
}
