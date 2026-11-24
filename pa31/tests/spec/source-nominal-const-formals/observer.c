#include <stdint.h>
#include <stdio.h>

extern uint8_t observed_enum(void);
extern uint8_t observed_record(void);

int main(void) {
    printf("%u %u\n", (unsigned)observed_enum(), (unsigned)observed_record());
    return 0;
}
