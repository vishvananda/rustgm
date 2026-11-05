#include <stdio.h>
#include <stdint.h>
void run_drop(uint32_t);void observe_drop(uint32_t value){printf("drop=%u\n",value);}int main(void){run_drop(31);}
