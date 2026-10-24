#include <stdio.h>
extern const unsigned char *answer(void);
extern const unsigned char *runtime(void);
int main(void){printf("%u %u\n",(unsigned)*answer(),(unsigned)*runtime());return 0;}
