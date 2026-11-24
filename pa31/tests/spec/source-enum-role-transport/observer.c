#include <stdbool.h>
#include <stdio.h>
extern unsigned char observed_option(bool input);
extern unsigned char observed_result(bool input);
extern signed char observed_ordering(void);
int main(void) {
    printf("%u %u %u %u %d\n", observed_option(true), observed_option(false),
        observed_result(true), observed_result(false), observed_ordering());
    return 0;
}
