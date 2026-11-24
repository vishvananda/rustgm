#include <stdint.h>
#include <stdio.h>
extern uint8_t observed_start(void);
extern uint8_t observed_end(void);
extern uint8_t observed_inclusive_start(void);
extern uint8_t observed_inclusive_end(void);
int main(void) {
    printf("%u %u %u %u\n", observed_start(), observed_end(),
           observed_inclusive_start(), observed_inclusive_end());
    return 0;
}
