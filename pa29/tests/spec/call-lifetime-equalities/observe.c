#include <stdint.h>

extern uint8_t absent(void);
extern uint8_t present(uint8_t value);

int main(void) {
    return absent() == 7 && present(26) == 26 && present(43) == 43 ? 0 : 1;
}
