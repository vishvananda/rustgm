#include <stdint.h>
extern uint8_t observe(uint8_t value);
int main(void) { return observe(37) == 37 && observe(73) == 73 ? 0 : 1; }
