#include <stdint.h>
#include <inttypes.h>
#include <stdio.h>

extern uint16_t observe_external(void);
extern intptr_t observe_local(void);
extern uint8_t observe_record(void);
int main(void) {
    printf("%u\n%" PRIdPTR "\n%u\n", (unsigned)observe_external(), observe_local(),
           (unsigned)observe_record());
}
