#include <stdio.h>
extern unsigned long observed(void);
void report_value(unsigned long value){printf("%lu\n",value);}
int main(void){report_value(observed());return 0;}
