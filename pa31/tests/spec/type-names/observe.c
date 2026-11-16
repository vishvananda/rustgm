#include <stddef.h>
#include <stdio.h>
struct text { const char *data; size_t length; };
extern struct text NAMES[26];
extern struct text GENERIC, FILLED;
void fill(void);
static void show(struct text name) { printf("%.*s\n", (int)name.length, name.data); }
int main(void) {
    fill();
    for (int i = 0; i < 26; ++i) show(NAMES[i]);
    show(GENERIC);
    show(FILLED);
    return 0;
}
