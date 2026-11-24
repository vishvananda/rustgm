#include <stddef.h>
#include <stdio.h>

double helper_diff(double a, double b) {
    return a - b;
}

extern size_t observe_len(void);
extern double observe_diff(void);

int main(void) {
    printf("%zu %.2f\n", observe_len(), observe_diff());
    return 0;
}
