#include <stdint.h>
#include <stdio.h>

extern const uint8_t *observed(const uint8_t *, const uint8_t *);
extern const uint8_t *observed_captured(const uint8_t *, const uint8_t *);

int main(void) {
    const uint8_t left = 19;
    const uint8_t right = 73;
    const uint8_t *member = observed(&left, &right);
    const uint8_t *capture = observed_captured(&left, &right);
    if (member != &left || capture != &right)
        return 1;
    printf("%u %u\n", *member, *capture);
    return 0;
}
