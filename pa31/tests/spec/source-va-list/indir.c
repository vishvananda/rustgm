#include <stdio.h>
struct Cursor{int first,second;};extern int va_indirect(struct Cursor*);int main(void){struct Cursor c={11,17};printf("%d %d\n",va_indirect(&c),c.first);return 0;}
