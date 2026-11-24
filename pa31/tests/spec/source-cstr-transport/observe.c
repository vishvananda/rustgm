#include <stdint.h>
#include <stdio.h>

extern uint8_t observed(void);

int main(void) {
    printf("%u\n", observed());
    return 0;
}

uint8_t check_bytes(const char *pointer) {
    return pointer[0] == '4' && pointer[1] == '1' && pointer[2] == 0 ? 41 : 0;
}
