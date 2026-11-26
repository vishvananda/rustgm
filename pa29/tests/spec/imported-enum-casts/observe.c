#include <stdint.h>

extern uint8_t explicit(void);
extern uint8_t implicit(void);
extern uint8_t empty_tuple(void);
extern uint8_t empty_record(void);

int main(void) {
    return explicit() == 16 && implicit() == 17 && empty_tuple() == 0
        && empty_record() == 1 ? 0 : 1;
}
