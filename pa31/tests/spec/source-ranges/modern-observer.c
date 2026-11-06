#include <stdio.h>
extern unsigned char modern_start(void),modern_last(void),from_start(void),to_end(void),to_last(void),full_value(void);int main(void){printf("%u %u %u %u %u %u\n",modern_start(),modern_last(),from_start(),to_end(),to_last(),full_value());}
