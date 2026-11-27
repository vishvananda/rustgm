#include <stdint.h>
#include <stdio.h>

extern uint8_t local_constant(void);
extern uint8_t checked_constructor(uint8_t value);
extern uint8_t stable_first(void);
extern uint8_t stable_second(void);
extern uint8_t ordinary_trait_member(void);
extern uint8_t nested_initializer(void);
extern uint8_t derived_implementation(void);

int main(void) {
    printf("%u %u %u %u %u %u %u %u\n", (unsigned)local_constant(), (unsigned)checked_constructor(34),
           (unsigned)checked_constructor(58), (unsigned)stable_first(),
           (unsigned)stable_second(), (unsigned)ordinary_trait_member(),
           (unsigned)nested_initializer(), (unsigned)derived_implementation());
    return 0;
}
