#include <stdint.h>
#include <stdio.h>

extern const uint8_t *observed(const uint8_t *);

int main(void) {
    uint8_t value = 19;
    const uint8_t *returned = observed(&value);
    if (returned != &value) {
        return 1;
    }
    printf("%u\n", (unsigned)*returned);
    return 0;
}
