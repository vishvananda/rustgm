#include <stdint.h>

extern uint8_t observed(uint8_t input);

int main(void) {
    return observed(23) == 23 && observed(59) == 59 ? 0 : 1;
}
