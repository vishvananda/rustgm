#include <stdio.h>
extern unsigned char observed_docs(void);
extern unsigned char observed_pointer(void);
int main(void){printf("%u %u\n",observed_docs(),observed_pointer());return 0;}
