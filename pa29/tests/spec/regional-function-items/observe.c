#include <stdint.h>

extern uint8_t observed(uint8_t left, uint8_t right);
extern uint8_t other_instance(uint8_t left, uint8_t right);

int main(void) {
    return observed(11, 31) == 20 && other_instance(12, 7) == 11 ? 0 : 1;
}
