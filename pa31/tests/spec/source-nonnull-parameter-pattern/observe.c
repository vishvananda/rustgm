#include <stdint.h>
#include <stdio.h>

extern uint8_t observed_nonnull_dispatch(void);
extern uint8_t observed_client_dispatch(void);

int main(void) {
    printf("%u %u\n", (unsigned)observed_nonnull_dispatch(), (unsigned)observed_client_dispatch());
    return 0;
}
