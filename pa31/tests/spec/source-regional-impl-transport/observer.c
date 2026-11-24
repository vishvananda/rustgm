#include <stdio.h>

extern unsigned char observed_bound(void);
extern unsigned char observed_method(void);
extern unsigned char observed_trait_lifetime(void);

int main(void) {
    printf("%u %u %u\n", (unsigned)observed_bound(), (unsigned)observed_method(),
           (unsigned)observed_trait_lifetime());
    return 0;
}
