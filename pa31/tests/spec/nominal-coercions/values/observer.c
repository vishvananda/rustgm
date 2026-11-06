#include <stdio.h>
extern unsigned char pin_value(void),prefix_value(void),nested_value(void),effect_value(void),owned_value(void),phantom_value(void);
static unsigned char events[32];static unsigned count;
void record(unsigned char tag){if(count<32)events[count++]=tag;}
static void observe(unsigned char(*f)(void)){count=0;unsigned value=f();printf("%u|",value);for(unsigned i=0;i<count;i++)printf("%s%u",i?",":"",events[i]);puts("");}
int main(void){observe(pin_value);observe(prefix_value);observe(nested_value);observe(effect_value);observe(owned_value);observe(phantom_value);return 0;}
