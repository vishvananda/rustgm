#include <stdint.h>
#include <stdio.h>
#include <string.h>

extern uint32_t answer(uint32_t *first);
uint32_t tick(uint32_t which) {
    printf("e%u;", which);
    return which == 1 ? 7 : 9;
}
void gone(uint32_t value) { printf("d%u;", value); }
void read_seen(uint32_t first, uint32_t second) {
    printf("r%u,%u;", first, second);
}
void template_seen(const unsigned char *bytes) {
    static const unsigned char expected[] = {
        0xc0, 7, ' ', '(', '1', ' ', '<', '<', ' ', 0xc0, 1, ')', 0
    };
    printf("t%d;", memcmp(bytes, expected, sizeof expected) == 0);
}
int main(void) {
    uint32_t first = 0;
    uint32_t second = answer(&first);
    printf("%u %u\n", first, second);
    return first == 7 && second == 9 ? 0 : 1;
}
