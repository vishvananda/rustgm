#include <stdint.h>
#include <stdio.h>
extern uint8_t observed_field(void);
extern uint8_t observed_signature(void);
extern uint8_t observed_higher_ranked(void);
int main(void) {
    printf("%u %u %u\n", observed_field(), observed_signature(), observed_higher_ranked());
    return 0;
}
