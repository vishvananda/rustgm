#include <stdio.h>
static unsigned calls;void*mark(void){++calls;return 0;}extern unsigned long answer(void);int main(void){unsigned long value=answer();printf("%lu %u\n",value,calls);return 0;}
