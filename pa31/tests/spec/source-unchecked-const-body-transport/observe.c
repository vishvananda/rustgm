#include <stdint.h>
#include <stdio.h>

extern uint32_t obs_free(void);
extern uint32_t obs_generic(void);
extern uint32_t obs_method(void);
extern uint32_t obs_const_dead(void);
extern uint32_t obs_comptime(void);
extern uint32_t obs_cmp(void);

int main(void) {
    printf(
        "%u %u %u %u %u %u\n",
        obs_free(),
        obs_generic(),
        obs_method(),
        obs_const_dead(),
        obs_comptime(),
        obs_cmp()
    );
    return 0;
}
