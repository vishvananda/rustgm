#include <stdint.h>
#include <stdio.h>
static unsigned trace;
static const uint8_t data[3]={7,8,9};
const uint8_t *observed_data(void){trace=trace*10+1;return data;}
uintptr_t observed_meta(void){trace=trace*10+2;return 3;}
const uint8_t *observed_thin_data(void){trace=trace*10+3;return data;}
void observed_unit(void){trace=trace*10+4;}
extern uintptr_t run_slice_len(void);
extern uint8_t run_thin(void);
int main(void){uintptr_t n=run_slice_len();unsigned first=trace;trace=0;unsigned v=run_thin();printf("%lu %u %u %u\n",(unsigned long)n,first,v,trace);return 0;}
