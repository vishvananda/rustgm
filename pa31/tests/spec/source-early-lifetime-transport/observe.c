#include <stdint.h>
#include <stdio.h>

extern void observe_early_lifetimes(uint8_t out[11]);

int main(void) {
    uint8_t o[11] = {0};
    observe_early_lifetimes(o);
    printf("%u %u %u %u %u %u %u %u %u %u %u\n",
           o[0], o[1], o[2], o[3], o[4], o[5],
           o[6], o[7], o[8], o[9], o[10]);
    return 0;
}
