#include <stdint.h>
#include <inttypes.h>
#include <stdio.h>
extern uintptr_t observed_size(void);
extern uintptr_t observed_alignment(void);
int main(void) {
    printf("%" PRIuPTR "\n%" PRIuPTR "\n", observed_size(), observed_alignment());
}
