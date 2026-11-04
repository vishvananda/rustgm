#include <stdio.h>
extern unsigned char dot_cloned(void);
extern unsigned char dot_copied(void);
extern unsigned char dot_moved(void);
extern unsigned char generic_bound(void);
extern unsigned char generic_open(void);
extern unsigned char closure_cloned(void);
extern unsigned char closure_copied(void);
extern unsigned char closure_moved(void);
int main(void){printf("%u\n",dot_cloned());printf("%u\n",dot_copied());printf("%u\n",dot_moved());printf("%u\n",generic_bound());printf("%u\n",generic_open());printf("%u\n",closure_cloned());printf("%u\n",closure_copied());printf("%u\n",closure_moved());return 0;}
