#include <stdarg.h>
#include <stdio.h>
#undef va_copy
extern int va_int(int,...),va_copy(int,...),va_stack(int,...),va_early(int,...),va_owned(void*);
extern double va_float(int,...);extern const unsigned char*va_ptr(int,...);
static int owned(int first,...){va_list a;va_start(a,first);int n=va_owned(a);va_end(a);return n;}
int main(void){unsigned char point=23;printf("%d %d %.1f %u %d %d %d\n",va_int(0,11,17),va_copy(0,11,17),va_float(0,11.5,17.5),(unsigned)*va_ptr(0,&point),va_stack(0,1,2,3,4,5,6,7,23),va_early(0,29),owned(0,31));return 0;}
