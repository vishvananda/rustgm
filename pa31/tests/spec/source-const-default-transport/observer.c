#include <stdint.h>
#include <stdio.h>

extern uint8_t observed_scalar(void);
extern uint8_t observed_record(void);

int main(void) {
    printf("%u %u\n", (unsigned)observed_scalar(), (unsigned)observed_record());
    return 0;
}
