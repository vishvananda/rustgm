#include <stdio.h>
extern const signed char *round_raw(void);
extern const unsigned char *round_view(void);
int main(void){const unsigned char *a=(const unsigned char*)round_raw(),*b=round_view();if(a[0]!=255||a[1]||b[0]!=255||b[1])return 1;printf("roundtrip %u %u\n",a[0],b[1]);return 0;}
