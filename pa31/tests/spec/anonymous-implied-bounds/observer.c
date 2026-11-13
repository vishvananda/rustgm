#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
extern uint8_t observe(void);int main(void){printf("%u\n",observe());return 0;}
void host_panic(const uint8_t *data,size_t length){fwrite(data,1,length,stderr);abort();}
