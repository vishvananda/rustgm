#include <stdio.h>
extern int observed_off(void);extern int observed_on(void);
int main(void){printf("%d %d\n",observed_off(),observed_on());}
