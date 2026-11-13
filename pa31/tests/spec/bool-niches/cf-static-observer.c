#include <stdint.h>
#include <stdio.h>
extern const uint8_t STOP,GO;extern uint8_t get_stop(void),get_go(void);
int main(void){printf("%u %u %u %u\n",STOP,GO,get_stop(),get_go());return 0;}
