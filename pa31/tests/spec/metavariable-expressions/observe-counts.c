#include <stddef.h>
#include <stdio.h>

extern size_t observed_flat(void);
extern size_t observed_nested(void);
extern size_t observed_outer(void);
extern size_t observed_selected(void);

int main(void) {
    printf("%zu %zu %zu %zu\n", observed_flat(), observed_nested(),
           observed_outer(), observed_selected());
    return 0;
}
