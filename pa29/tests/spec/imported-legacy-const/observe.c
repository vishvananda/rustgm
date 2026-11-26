#include <stddef.h>

extern size_t legacy(void);
extern size_t legacy_runtime(size_t, size_t);
extern size_t modern(void);
extern size_t alias(void);

int main(void) {
    return legacy() == 113 && legacy_runtime(9, 3) == 150
        && modern() == 112 && alias() == 66 ? 0 : 1;
}
