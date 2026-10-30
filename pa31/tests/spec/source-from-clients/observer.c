#include <stdint.h>
#include <stdio.h>
extern uint8_t imported_observed(void);
int main(void) {
    printf("%u\n", (unsigned)imported_observed());
    return 0;
}
