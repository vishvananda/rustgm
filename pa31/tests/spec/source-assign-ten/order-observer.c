#include <stdio.h>
int order_value(int);int order_state(int);int main(void){volatile int seed=0;printf("%d %d\n",order_value(seed),order_state(seed));return 0;}
