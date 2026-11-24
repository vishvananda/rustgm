#include <stdint.h>
#include <stdio.h>

extern uint8_t observed(uint8_t which);

int main(void) {
    printf("%u %u\n", observed(0), observed(1));
    return 0;
}
