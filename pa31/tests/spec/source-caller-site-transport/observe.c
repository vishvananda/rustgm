#include <stdint.h>
#include <stdio.h>

extern uint32_t chained_native_line(void);
extern uint32_t chained_const_line(void);
extern uint32_t untracked_native_line(void);
extern uint32_t untracked_const_line(void);
extern uint32_t branch_native_a(void);
extern uint32_t branch_native_b(void);
extern uint32_t branch_const_a(void);
extern uint32_t branch_const_b(void);
extern uint32_t trait_tracked_line(void);
extern uint32_t trait_untracked_line(void);

int main(void) {
    printf("%u %u %u %u %u %u %u %u %u %u\n",
           chained_native_line(),
           chained_const_line(),
           untracked_native_line(),
           untracked_const_line(),
           branch_native_a(),
           branch_native_b(),
           branch_const_a(),
           branch_const_b(),
           trait_tracked_line(),
           trait_untracked_line());
    return 0;
}
