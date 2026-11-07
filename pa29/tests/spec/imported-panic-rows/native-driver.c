#include <stdio.h>
extern unsigned explicit(void);
extern unsigned unreached(void);
extern unsigned unreached_formatted(void);
extern unsigned skipped(void);
int main(void) {
    printf("%u %u %u %u\n", explicit(), unreached(), unreached_formatted(), skipped());
    return 0;
}
