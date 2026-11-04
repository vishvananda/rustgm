#include <stdio.h>
extern unsigned char observed_cfg(void);
extern unsigned char observed_notable(void);
int main(void){printf("%u %u\n",observed_cfg(),observed_notable());return 0;}
