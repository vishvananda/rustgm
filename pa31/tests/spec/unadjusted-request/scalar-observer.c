#include <stdio.h>
extern unsigned char observed_byte(unsigned char);
extern unsigned long observed_word(unsigned long);
extern unsigned __int128 observed_wide(unsigned __int128);
extern float observed_single(float);
extern double observed_double(double);
extern _Bool observed_bit(_Bool);
int main(void){unsigned __int128 c=((unsigned __int128)1<<127)|1;printf("%u %lu %016lx%016lx %.2f %.2f %u\n",observed_byte(255),observed_word(~0UL),(unsigned long)(observed_wide(c)>>64),(unsigned long)observed_wide(c),(double)observed_single(1.5f),observed_double(-2.25),observed_bit(1));return 0;}
