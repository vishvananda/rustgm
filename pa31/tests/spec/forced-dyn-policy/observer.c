#include <stdio.h>
extern unsigned char selected(void);
extern unsigned char ordinary(void);
int main(void) {
    printf("%u\n%u\n", (unsigned)selected(), (unsigned)ordinary());
    return 0;
}
