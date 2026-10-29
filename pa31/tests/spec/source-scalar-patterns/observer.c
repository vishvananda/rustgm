#include <stdint.h>
#include <stdio.h>
extern uint32_t observed_low(void),observed_mid(void),observed_high(void),observed_char(void);
extern int32_t observed_signed(void);
int main(void){printf("%u %u %u %d %u\n",observed_low(),observed_mid(),observed_high(),observed_signed(),observed_char());return 0;}
