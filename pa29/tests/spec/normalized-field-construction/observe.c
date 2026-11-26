#include <stdint.h>

extern uint8_t built(uint8_t value);

int main(void) {
    return built(24) == 31 && built(51) == 52 ? 0 : 1;
}
