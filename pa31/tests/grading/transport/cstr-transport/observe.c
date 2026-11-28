#include <stdint.h>
extern uint8_t observed(void);
uint8_t check_bytes(const char *p) { return p[0] == '4' && p[1] == '1' && p[2] == 0 ? 41 : 0; }
int main(void) { return observed() == 41 ? 0 : 1; }
