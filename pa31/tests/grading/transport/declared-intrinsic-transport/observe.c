#include <stdint.h>
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
  return observe_comptime_free() == 41 && observe_comptime_method() == 77
      && observe_ctpop() == 4 && observe_transmute() == 19 && observe_black_box() == 23
      && observe_unchecked_sub() == 19 && observe_sub_ok() == 13 && observe_sub_overflow() == 100
      && observe_select() == 55 && observe_hinted() == 42 ? 0 : 1;
}
