#include <stdint.h>
#include <stdio.h>

extern uint32_t observe_comptime_free(void);
extern uint32_t observe_comptime_method(void);
extern uint32_t observe_ctpop(void);
extern uint32_t observe_transmute(void);
extern uint32_t observe_black_box(void);
extern uint32_t observe_unchecked_sub(void);
extern uint32_t observe_sub_ok(void);
extern uint32_t observe_sub_overflow(void);
extern uint32_t observe_select(void);
extern uint32_t observe_hinted(void);

int main(void) {
    printf(
        "%u %u %u %u %u %u %u %u %u %u\n",
        observe_comptime_free(),
        observe_comptime_method(),
        observe_ctpop(),
        observe_transmute(),
        observe_black_box(),
        observe_unchecked_sub(),
        observe_sub_ok(),
        observe_sub_overflow(),
        observe_select(),
        observe_hinted()
    );
    return 0;
}
