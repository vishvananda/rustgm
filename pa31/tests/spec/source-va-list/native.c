#include <stdarg.h>
#include <stdio.h>
#include <stddef.h>
extern int va_read(int,...);
extern size_t va_layout(void*);
static size_t layout(int unused,...){va_list a;va_start(a,unused);size_t n=va_layout(a);va_end(a);return n;}
int main(void){int a=11,b=17;printf("%d %zu\n",va_read(0,a,b),layout(0,a,b));return 0;}
