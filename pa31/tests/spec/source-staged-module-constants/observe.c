#include <stdint.h>
#include <stdio.h>

extern uint8_t stable_value(void);
extern uint8_t experimental_value(void);
extern uint8_t nested_value(void);

int main(void) {
    printf("%u %u %u\n", (unsigned)stable_value(), (unsigned)experimental_value(),
           (unsigned)nested_value());
    return 0;
}
