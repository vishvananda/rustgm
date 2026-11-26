#include <stdint.h>

extern uint8_t observed(uint8_t value, uint8_t mask);

int main(void) {
    return observed(42, 1) == 43 && observed(7, 3) == 4 ? 0 : 1;
}
