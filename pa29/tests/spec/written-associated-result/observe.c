#include <stdint.h>

extern uint8_t observe(uint8_t);

int main(void) {
    unsigned first = observe(37);
    unsigned second = observe(73);
    return first != 37 || second != 73;
}
