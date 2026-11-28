#include <stdint.h>
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
  return observe_const_fixed() == 5 && observe_const_doubled() == 10
      && observe_const_tag() == 11 && observe_const_unsafe() == 8 && observe_const_ref() == 7
      && observe_const_strict() == 4 && observe_const_twice() == 12 && observe_dyn_factor() == 9
      && observe_dyn_doubled() == 18 && observe_fixed_bonus() == 105 && observe_dyn_bonus() == 109 ? 0 : 1;
}
