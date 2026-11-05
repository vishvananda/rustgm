#include <stdint.h>
#include <stdio.h>
extern void run(int32_t *);
int main(void){int32_t seen=0;run(&seen);printf("%d\n",seen);return seen==41?0:1;}
