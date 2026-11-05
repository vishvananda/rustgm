#include <stdio.h>
extern int va_from_rust(void);int main(void){printf("%d\n",va_from_rust());return 0;}
