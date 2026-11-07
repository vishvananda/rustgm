#include <stdio.h>
extern unsigned unreached(void);
extern unsigned unreached_formatted(void);
int main(void) {
    printf("%u %u\n", unreached(), unreached_formatted());
    return 0;
}
