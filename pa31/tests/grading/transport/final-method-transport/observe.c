#include <stdint.h>
extern uint8_t observed_final(void);
extern uint8_t observed_override(void);
int main(void) { return observed_final() == 41 && observed_override() == 73 ? 0 : 1; }
