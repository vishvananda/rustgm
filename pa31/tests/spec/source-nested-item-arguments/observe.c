#include <stdint.h>
#include <stdio.h>

extern uint32_t obs_tuple(void);
extern uint32_t obs_nominal(void);
extern uint32_t obs_array(void);
extern uint32_t obs_reference(void);
extern uint32_t obs_lifetime(void);
extern uint32_t obs_other_item(void);

int main(void) {
    printf("%u %u %u %u %u %u\n", obs_tuple(), obs_nominal(), obs_array(),
           obs_reference(), obs_lifetime(), obs_other_item());
    return 0;
}
