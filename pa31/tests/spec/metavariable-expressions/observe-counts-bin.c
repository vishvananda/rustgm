#include <stddef.h>
#include <stdio.h>

extern size_t observed_flat(void);
extern size_t observed_nested(void);
extern size_t observed_outer(void);
extern size_t observed_selected(void);

void observe_counts(size_t a, size_t b, size_t c, size_t d) {
    printf("%zu %zu %zu %zu\n", a, b, c, d);
}

#ifndef RUSTGM_CANDIDATE
int main(void) {
    observe_counts(observed_flat(), observed_nested(), observed_outer(),
                   observed_selected());
    return 0;
}
#endif
