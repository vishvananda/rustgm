#include <stdint.h>

extern uint8_t shared(uint8_t value);
extern uint8_t exclusive(uint8_t value);

int main(void) {
    return shared(13) == 13 && shared(29) == 29
        && exclusive(16) == 17 && exclusive(25) == 24 ? 0 : 1;
}
