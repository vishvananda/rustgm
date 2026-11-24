#include <stdint.h>
#include <stdio.h>
extern uint8_t observed_context(void);
extern uint8_t observed_hooks(void);
extern uint8_t observed_borrowed_fn(void);
int main(void) {
    printf("%u %u %u\n", observed_context(), observed_hooks(), observed_borrowed_fn());
    return 0;
}
