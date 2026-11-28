#include <stdint.h>
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
  return chained_native_line() == 12 && chained_const_line() == 5
      && untracked_native_line() == 126 && untracked_const_line() == 126
      && branch_native_a() == 131 && branch_native_b() == 133
      && branch_const_a() == 131 && branch_const_b() == 133
      && trait_tracked_line() == 158 && trait_untracked_line() == 153 ? 0 : 1;
}
