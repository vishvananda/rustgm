#include <stdint.h>

extern uint8_t observed(uint8_t left, uint8_t right);

int main(void) {
    return observed(7, 0) == 7 && observed(1, 42) == 42 ? 0 : 1;
}
