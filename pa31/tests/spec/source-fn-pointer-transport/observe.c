#include <stdint.h>
#include <stdio.h>

extern uint8_t observed(void);

int main(void) {
    printf("%u\n", observed());
    return 0;
}

static uint8_t callback(void) {
    return 41;
}

uint8_t (*provide_pointer(void))(void) {
    return callback;
}

uint8_t observe_pointer(const void *address) {
    return address == (const void *)callback ? callback() : 0;
}
