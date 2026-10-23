#include <stdio.h>
static unsigned count=0;static unsigned char seen[256];void record(unsigned char n){if(count<256)seen[count]=n;count++;}
extern void observe(void*);int main(void){unsigned char storage=1;observe(&storage);printf("%u\n",count);if(count!=2)return 1;unsigned char expected[]={1,3};for(unsigned i=0;i<count;i++)if(seen[i]!=expected[i])return 2;return 0;}
