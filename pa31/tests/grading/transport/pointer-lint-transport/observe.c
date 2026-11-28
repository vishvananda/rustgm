#include <stdint.h>
extern uint8_t observed_free(uint8_t value);
extern uint8_t observed_method(uint8_t value);
int main(void) { return observed_free(41) == 41 && observed_method(73) == 73 ? 0 : 1; }
