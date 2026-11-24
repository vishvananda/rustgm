#include <stddef.h>
#include <stdio.h>

size_t direct_size(void);
size_t direct_alignment(void);
size_t default_size(void);
size_t default_alignment(void);
size_t runtime_size(void);
size_t runtime_alignment(void);

int main(void) {
    printf("%zu %zu %zu %zu %zu %zu\n", direct_size(), direct_alignment(), default_size(),
        default_alignment(), runtime_size(), runtime_alignment());
    return 0;
}
