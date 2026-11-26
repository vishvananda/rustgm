#include <stdint.h>

extern uint8_t observed(uint8_t value);

int main(void) {
    return observed(7) == 7 && observed(42) == 42 ? 0 : 1;
}
