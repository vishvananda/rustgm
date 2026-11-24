#include <stdint.h>
#include <stdio.h>

extern uint32_t observe_const_fixed(void);
extern uint32_t observe_const_doubled(void);
extern uint32_t observe_const_tag(void);
extern uint32_t observe_const_unsafe(void);
extern uint32_t observe_const_ref(void);
extern uint32_t observe_const_strict(void);
extern uint32_t observe_const_twice(void);
extern uint32_t observe_dyn_factor(void);
extern uint32_t observe_dyn_doubled(void);
extern uint32_t observe_fixed_bonus(void);
extern uint32_t observe_dyn_bonus(void);

int main(void) {
    printf(
        "%u %u %u %u %u %u %u %u %u %u %u\n",
        observe_const_fixed(),
        observe_const_doubled(),
        observe_const_tag(),
        observe_const_unsafe(),
        observe_const_ref(),
        observe_const_strict(),
        observe_const_twice(),
        observe_dyn_factor(),
        observe_dyn_doubled(),
        observe_fixed_bonus(),
        observe_dyn_bonus()
    );
    return 0;
}
