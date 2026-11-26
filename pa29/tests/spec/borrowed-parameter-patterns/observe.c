#include <stdint.h>

extern uint8_t plain(uint8_t input);
extern uint8_t field(uint8_t input);

int main(void) {
    return plain(24) == 31 && field(46) == 41 ? 0 : 1;
}
