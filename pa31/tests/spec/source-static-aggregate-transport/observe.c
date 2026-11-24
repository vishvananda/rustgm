#include <stdint.h>
#include <stdio.h>
extern uint32_t observed_headers(void);
extern uint8_t observed_digit(void);
extern uint8_t observed_table(void);
extern uint8_t observed_vtable(void);
int main(void) {
    printf("%u %u %u %u\n", observed_headers(), observed_digit(), observed_table(), observed_vtable());
    return 0;
}
