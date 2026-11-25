#include <stdint.h>
#include <stdio.h>

extern uint32_t obs_shifts(void);
extern uint32_t obs_ptr_union(void);
extern uint32_t obs_repeat(void);
extern uint32_t obs_slice_text(void);
extern uint32_t obs_casts(void);

int main(void) {
    printf("%u %u %u %u %u\n",
           obs_shifts(),
           obs_ptr_union(),
           obs_repeat(),
           obs_slice_text(),
           obs_casts());
    return 0;
}
