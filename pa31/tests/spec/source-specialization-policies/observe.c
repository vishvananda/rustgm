#include <stdint.h>

extern uint8_t applicable_blanket(void);
extern uint8_t applicable_specialized(void);
extern uint8_t trusted_blanket(void);
extern uint8_t trusted_specialized(void);

int main(void) {
    return applicable_blanket() == 1 && applicable_specialized() == 2
        && trusted_blanket() == 3 && trusted_specialized() == 4 ? 0 : 1;
}
