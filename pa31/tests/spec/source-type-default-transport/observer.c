#include <stdint.h>
#include <stdio.h>

extern uint8_t observed_flow(void);
extern uint8_t observed_lazy(void);
extern uint8_t observed_closed(void);
extern uint8_t observed_projection(void);

int main(void) {
    printf("%u %u %u %u\n",
           (unsigned)observed_flow(), (unsigned)observed_lazy(),
           (unsigned)observed_closed(), (unsigned)observed_projection());
    return 0;
}
