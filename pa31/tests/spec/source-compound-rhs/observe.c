#include <stdio.h>
extern unsigned char run(void);
void host_left(void) { puts("left"); }
void host_rhs(void) { puts("rhs"); }
void host_deref(void) { puts("deref"); }
void host_mut(void) { puts("mut"); }
void host_add(void) { puts("add"); }
void host_assign(void) { puts("assign"); }
int main(void) { unsigned value = run(); printf("value %u\n", value); return 0; }
