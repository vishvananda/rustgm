#include <stdint.h>

extern uint8_t field_observation(uint8_t input);
extern uint8_t array_observation(uint8_t left, uint8_t right);

int main(void) {
    return field_observation(12) == 14 && field_observation(19) == 17 && array_observation(17, 44) == 61 ? 0 : 1;
}
