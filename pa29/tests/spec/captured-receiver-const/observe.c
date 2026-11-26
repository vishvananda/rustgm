#include <stdint.h>

extern uint8_t read_three(uint8_t value);
extern uint8_t read_four(uint8_t value);

int main(void) {
    return read_three(11) == 11 && read_three(29) == 29
        && read_four(5) == 7 ? 0 : 1;
}
