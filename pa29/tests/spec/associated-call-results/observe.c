#include <stdint.h>

extern uint8_t observed(uint8_t input);

int main(void) {
    return observed(25) == 25 && observed(47) == 47 ? 0 : 1;
}
