#include <stdio.h>
#include <stdbool.h>
extern bool range_entry(void);extern bool range_probe(unsigned char);int main(void){printf("%u %u %u %u\n",(unsigned)range_entry(),(unsigned)range_probe(2),(unsigned)range_probe(3),(unsigned)range_probe(7));return 0;}
