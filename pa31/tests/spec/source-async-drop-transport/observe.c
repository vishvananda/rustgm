#include <stdint.h>
#include <stdio.h>

extern uint32_t observe_ready_work(void);
extern uint32_t observe_poll_tag(void);

int main(void) {
    printf("%u %u\n", observe_ready_work(), observe_poll_tag());
    return 0;
}
