#include <stdint.h>

extern uint8_t shared(uint8_t input, uint8_t argument);
extern uint8_t update(uint8_t input);
extern uint8_t consume(uint8_t input, uint8_t argument);

int main(void) {
    return shared(11, 5) == 6 && shared(15, 9) == 10 && update(7) == 11
        && update(14) == 23 && consume(12, 5) == 11 && consume(18, 8) == 24
        ? 0 : 1;
}
