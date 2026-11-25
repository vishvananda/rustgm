#include <stdint.h>
#include <stdio.h>

extern uint32_t obs_direct(void);
extern uint32_t obs_forwarded(void);
extern uint32_t obs_first(void);
extern uint32_t obs_second(void);
extern uint32_t obs_method(void);
extern uint32_t obs_ordinary(void);

int main(void) {
    printf("%u %u %u %u %u %u\n", obs_direct(), obs_forwarded(), obs_first(),
           obs_second(), obs_method(), obs_ordinary());
    return 0;
}
