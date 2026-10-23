#include <stdio.h>
static unsigned drops=0,sum=0,calls=0;void record(const unsigned char*value){drops++;sum+=*value;}void chosen(void){calls++;}extern unsigned char answer(void);int main(void){unsigned char value=answer();printf("%u %u %u %u\n",value,drops,sum,calls);return 0;}
