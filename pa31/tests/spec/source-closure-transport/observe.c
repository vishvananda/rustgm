#include <stdint.h>
#include <stdio.h>

extern uint32_t obs_const_count(void);
extern uint32_t obs_reify(void);
extern uint32_t obs_capture_fn(void);
extern uint32_t obs_unique_mut(void);
extern uint32_t obs_once_move(void);
extern uint32_t obs_nested_ref(void);
extern uint32_t obs_rpit_closure(void);

int main(void) {
    printf(
        "%u %u %u %u %u %u %u\n",
        obs_const_count(),
        obs_reify(),
        obs_capture_fn(),
        obs_unique_mut(),
        obs_once_move(),
        obs_nested_ref(),
        obs_rpit_closure()
    );
    return 0;
}
