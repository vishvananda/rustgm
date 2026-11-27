#include <stdio.h>

extern void plain(void);
extern void with_newline(void);
extern void constant(void);

void observe_text(const char *text, size_t length) {
    fwrite(text, 1, length, stdout);
}

int main(void) {
    plain();
    with_newline();
    constant();
    return 0;
}
