#include <stdio.h>
static unsigned count=0;void record(unsigned char n){if(n==2)count++;else count=1000;}extern void observe(void*);int main(void){unsigned char storage[3]={1,2,3};observe(&storage);printf("%u\n",count);return 0;}
