#include <stdint.h>

extern uint8_t observed_default(void);
extern uint8_t observed_specialized(void);
extern uintptr_t observed_inherited_size(void);
extern uint16_t observed_inherited_type(void);

int main(void) {
    return observed_default() == 1 && observed_specialized() == 2
        && observed_inherited_size() == 2 && observed_inherited_type() == 641 ? 0 : 1;
}
