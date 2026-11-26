#include <stdint.h>

extern uint8_t read_shared(uint8_t first, uint8_t second);
extern uint8_t read_unique(uint8_t first, uint8_t second);

int main(void) {
    return read_shared(17, 39) == 39 && read_shared(4, 39) == 4
        && read_unique(25, 41) == 41 && read_unique(8, 41) == 8 ? 0 : 1;
}
