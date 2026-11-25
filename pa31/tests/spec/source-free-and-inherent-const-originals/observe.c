#include <stdint.h>
#include <stdio.h>

extern void observe_const_originals(uint32_t *out);

int main(void) {
    uint32_t out[11] = {0};
    observe_const_originals(out);
    printf("%u %u %u %u %u %u %u %u %u %u %u\n",
           out[0], out[1], out[2], out[3], out[4], out[5],
           out[6], out[7], out[8], out[9], out[10]);
    return 0;
}
