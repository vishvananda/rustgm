#include <stdint.h>
#include <stdio.h>

extern uint32_t observed_cast_lane(void);
extern uint32_t observed_mixed_view(void);
extern uint32_t observed_ptr_lane(void);

int main(void) {
    printf("%u %u %u\n",
           observed_cast_lane(),
           observed_mixed_view(),
           observed_ptr_lane());
    return 0;
}
