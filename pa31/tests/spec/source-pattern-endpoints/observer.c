#include <stdint.h>
#include <stdio.h>
extern uint8_t observed_upper(void),observed_lower(void),observed_exclusive(void);
extern int32_t observed_signed(void),observed_positive(void);
int main(void){printf("%u %u %u %d %d\n",observed_upper(),observed_lower(),observed_exclusive(),observed_signed(),observed_positive());return 0;}
