#include <stdint.h>
#include <stdio.h>

extern uint32_t obs_for_concrete(void);
extern uint32_t obs_for_generic(void);
extern uint32_t obs_try_some(void);
extern uint32_t obs_try_none(void);
extern uint32_t obs_deref_all(void);
extern uint32_t obs_offset(void);

int main(void) {
    printf(
        "%u %u %u %u %u %u\n",
        obs_for_concrete(),
        obs_for_generic(),
        obs_try_some(),
        obs_try_none(),
        obs_deref_all(),
        obs_offset()
    );
    return 0;
}
