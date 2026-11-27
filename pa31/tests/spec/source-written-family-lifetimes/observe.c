#include <stdint.h>
#include <stdio.h>

extern uint8_t observed(uint8_t);
int main(void) {
    printf("%u\n", observed(28));
    return 0;
}
