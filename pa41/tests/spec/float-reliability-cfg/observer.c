#include <stdio.h>
extern unsigned char observed_0(void);
extern unsigned char observed_1(void);
extern unsigned char observed_2(void);
extern unsigned char observed_3(void);
int main(void){printf("%u %u %u %u\n",observed_0(),observed_1(),observed_2(),observed_3());return 0;}
