#include <stdio.h>
extern unsigned char exclusive(unsigned char*,unsigned char),shared(unsigned char*,unsigned char),clone_count(void),explicit_clone(const unsigned char*),constant(void);
int main(void){unsigned char value=7;unsigned a=exclusive(&value,23),b=value,c=shared(&value,47),d=value,e=clone_count();unsigned f=explicit_clone(&value),g=clone_count(),k=constant();if(a!=23||b!=23||c!=47||d!=47||e!=0||f!=47||g!=99||k!=31)return 1;printf("reborrow %u %u %u %u %u %u %u\n",a,b,c,d,e,g,k);return 0;}
