#include <stdint.h>
#include <stdio.h>

extern uint32_t observed_bool(void);
extern uint32_t observed_u32(void);
extern uint32_t observed_char(void);
extern uint32_t observed_max(void);
extern uintptr_t observed_usize(void);

int main(void) {
    printf("%u %u %u %u %llu\n",
           observed_bool(),
           observed_u32(),
           observed_char(),
           observed_max(),
           (unsigned long long)observed_usize());
    return 0;
}
