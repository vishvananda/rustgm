#include <stdio.h>

unsigned nostd_prelude(void);
unsigned nostd_path(void);

int main(void) {
    printf("%u %u\n", nostd_prelude(), nostd_path());
    return 0;
}
