#include <stdint.h>
#include <stdio.h>
extern uint8_t observed_forward(void);
extern uint8_t observed_item(void);
int main(void) {
    unsigned forward = observed_forward();
    unsigned item = observed_item();
    printf("%u %u\n", forward, item);
    return forward != 7 || item != 9;
}
