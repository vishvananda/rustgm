#include <stdint.h>
extern uint8_t observed_start(void);
extern uint8_t observed_end(void);
extern uint8_t observed_inclusive_start(void);
extern uint8_t observed_inclusive_end(void);
int main(void) { return observed_start() == 8 && observed_end() == 16 && observed_inclusive_start() == 23 && observed_inclusive_end() == 47 ? 0 : 1; }
