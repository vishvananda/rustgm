#include <stdint.h>

extern uint8_t observed(uint8_t value, uint8_t mask);
extern uint8_t observed_shared(uint8_t value);

int main(void) {
    return observed(42, 1) == 43 && observed(7, 3) == 4
        && observed_shared(11) == 11 && observed_shared(19) == 19 ? 0 : 1;
}
