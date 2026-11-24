#include <stdint.h>
#include <stdio.h>
extern uint8_t observed_void(void);
extern uint8_t observed_dyn_inherent(void);
extern uint8_t observed_dyn_send_inherent(void);
int main(void) {
    printf("%u %u %u\n", observed_void(), observed_dyn_inherent(), observed_dyn_send_inherent());
    return 0;
}
