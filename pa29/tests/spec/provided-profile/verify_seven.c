#include <stdint.h>
#include <stdlib.h>

static int calls = 0;

static void check_done(void) {
    if (calls != 1) {
        abort();
    }
}

void observe(uint8_t value) {
    if (calls == 0) {
        atexit(check_done);
        if (value != 7u) {
            abort();
        }
    } else {
        abort();
    }
    calls += 1;
}
