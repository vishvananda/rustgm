#include <stdio.h>

extern unsigned char observed_continue(void);
extern unsigned char observed_break(void);

int main(void) {
    printf("%u %u\n", observed_continue(), observed_break());
    return 0;
}
