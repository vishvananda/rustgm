#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
extern uint8_t user_abort(void) __asm__("abort");
extern uint8_t probe(bool);
void marker(void) { puts("interposed"); fflush(stdout); }
int main(int argc, char **argv) {
    if (argc == 2 && argv[1][0] == 't') {
        puts("before"); fflush(stdout);
        const unsigned result = probe(true);
        printf("after %u\n", result); fflush(stdout);
        return 93;
    }
    const unsigned a = user_abort(), b = probe(false);
    printf("%u %u\n", a, b);
    return a == 7 && b == 9 ? 0 : 1;
}
