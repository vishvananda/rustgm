#include <stdint.h>

extern uint8_t first(uint8_t value);
extern uint8_t second(uint8_t value);
extern uint8_t values(void);

int main(void) {
    return first(11) == 8 && first(29) == 30
        && second(11) == 14 && second(29) == 24
        && values() == 12 ? 0 : 1;
}
