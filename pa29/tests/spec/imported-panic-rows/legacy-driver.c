#include <stdio.h>
extern unsigned unskipped(void);
int main(void) {
    printf("%u\n", unskipped());
    return 0;
}
