#include <stdint.h>

extern int32_t entry(uint8_t value);

int main(void) {
    return entry(29) == 42 && entry(87) == 43 ? 0 : 1;
}
