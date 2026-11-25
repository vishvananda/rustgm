#include <stdint.h>
#include <stdio.h>

extern uint32_t obs_temp(void);
extern uint32_t obs_bytes(void);
extern uint32_t obs_cstr(void);
extern uint32_t obs_slice(void);
extern uint32_t obs_object(void);
extern uint32_t obs_upcast(void);
extern uint32_t obs_bound(void);
extern uint32_t obs_nominal(void);

int main(void) {
    printf(
        "%u %u %u %u %u %u %u %u\n",
        obs_temp(),
        obs_bytes(),
        obs_cstr(),
        obs_slice(),
        obs_object(),
        obs_upcast(),
        obs_bound(),
        obs_nominal()
    );
    return 0;
}
