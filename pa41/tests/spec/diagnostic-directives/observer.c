#include <stdio.h>
extern unsigned char observed_trait(void);
extern unsigned char observed_macro(void);
int main(void){printf("%u %u\n",observed_trait(),observed_macro());return 0;}
