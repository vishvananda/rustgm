#include <stdio.h>
extern void try_drop(void);
static unsigned events;
void record(unsigned char value) { events = events * 10 + value; }
int main(void) {
    try_drop();
    printf("%u\n", events);
}
