#include <stdint.h>
#include <stdio.h>
extern void drop_value(uint8_t);
extern void drop_local_none(void);
extern void drop_local_some(void);
static unsigned count;
void record(uint8_t value){++count;printf("%u ",(unsigned)value);}
int main(void){printf("drop ");drop_value(0);if(count)return 1;drop_local_none();if(count)return 2;drop_value(1);if(count!=1)return 3;drop_value(255);if(count!=2)return 4;drop_local_some();if(count!=3)return 5;puts("done");return 0;}
