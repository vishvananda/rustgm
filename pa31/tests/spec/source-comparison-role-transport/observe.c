#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

extern bool observed_equal(uint8_t);
extern bool observed_unequal(uint8_t);
extern bool observed_different_rhs(uint8_t);
extern bool observed_less(uint8_t);
extern bool observed_greater_equal(uint8_t);

int main(void) {
    printf("%d %d %d %d %d %d\n",
           observed_equal(0), observed_equal(1),
           observed_unequal(0), observed_unequal(1),
           observed_different_rhs(0), observed_different_rhs(1));
    printf("%d %d %d %d  %d %d %d %d\n",
           observed_less(0), observed_less(1), observed_less(2), observed_less(3),
           observed_greater_equal(0), observed_greater_equal(1),
           observed_greater_equal(2), observed_greater_equal(3));
    return 0;
}
