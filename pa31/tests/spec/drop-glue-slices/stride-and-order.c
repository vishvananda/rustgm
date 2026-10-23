#include <stdio.h>
static unsigned count=0,failed=0;static unsigned char storage[3]={1,2,3};void record_address(unsigned char*p){if(count>=3||p!=storage+count)failed=1;count++;}extern void observe(void*);int main(void){observe(storage);printf("%u %u\n",count,failed);return 0;}
