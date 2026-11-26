#include <stdint.h>
#include <stdio.h>
extern uint32_t obs_select(void), obs_borrow(void), obs_value(void);
extern uint32_t obs_block(void), obs_check(void), obs_forward(void);
int main(void) {
    printf("%u %u %u %u %u %u\n", obs_select(), obs_borrow(), obs_value(),
           obs_block(), obs_check(), obs_forward());
    return 0;
}
