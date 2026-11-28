#include <stdint.h>
extern uint8_t observed(void);
int main(void) { return observed() == 41 ? 0 : 1; }
