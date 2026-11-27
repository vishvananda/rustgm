#include <inttypes.h>
#include <stdio.h>

extern uint64_t observed_quotient(uint64_t, uint64_t);
extern uint64_t observed_remainder(uint64_t, uint64_t);

int main(void) {
    printf("%" PRIu64 " %" PRIu64 "\n",
           observed_quotient(101, 8), observed_remainder(101, 8));
    return 0;
}
