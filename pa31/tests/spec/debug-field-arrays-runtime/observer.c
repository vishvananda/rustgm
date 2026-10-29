#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
void require_index(uintptr_t index, uintptr_t length) { if (index>=length) abort(); }
void trace_label(const unsigned char *data, uintptr_t length) { fwrite(data,1,length,stdout);putchar(':'); }
void trace_byte(uint8_t value) { printf("%u;",(unsigned)value); }
void trace_end(void) { putchar('|'); }
extern uint8_t observed(void);
int main(void) { uint8_t value=observed();printf("%u\n",(unsigned)value);return value==7?0:1; }
