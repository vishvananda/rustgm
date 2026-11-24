#include <stddef.h>
#include <stdio.h>
extern size_t observed_const(void);
extern size_t observed_runtime(void);
int main(void) {
    printf("%zu %zu\n", observed_const(), observed_runtime());
    return 0;
}
