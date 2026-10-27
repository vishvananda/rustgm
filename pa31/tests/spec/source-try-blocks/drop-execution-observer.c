#include <stdio.h>
static unsigned events;
void record(unsigned char value) { events = events * 10 + value; }
void finish(void) { printf("%u\n", events); }
#ifndef RUSTGM_CANDIDATE
extern void run(void);
int main(void) { run(); }
#endif
