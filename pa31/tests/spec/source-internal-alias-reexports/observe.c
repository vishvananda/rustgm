#include <stdint.h>
#include <stdio.h>

extern uint32_t observed_base(void);
extern uint32_t observed_first(void);
extern uint32_t observed_second(void);
extern uint32_t observed_static(void);

int main(void) {
    printf("%u %u %u %u\n",
           observed_base(),
           observed_first(),
           observed_second(),
           observed_static());
    return 0;
}
