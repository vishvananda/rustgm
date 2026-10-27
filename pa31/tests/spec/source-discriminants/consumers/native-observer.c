#include <stdint.h>
#include <stdio.h>

extern int8_t signed_tag(void) __asm__("signed");
extern uint16_t payload(void);
extern intptr_t none(void);
extern intptr_t some(void);
extern uint8_t effects(void);

int main(void) {
    printf("%d %u %ld %ld %u\n", (int)signed_tag(), (unsigned)payload(),
           (long)none(), (long)some(), (unsigned)effects());
    return 0;
}
