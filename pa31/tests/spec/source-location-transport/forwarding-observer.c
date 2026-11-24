#include <stdint.h>
#include <stdio.h>

uint32_t direct_line(void);
uint32_t direct_column(void);
uint32_t folded_line(void);
uint32_t folded_column(void);

int main(void) {
    printf("%u %u %u %u\n", direct_line(), direct_column(), folded_line(), folded_column());
    return 0;
}
