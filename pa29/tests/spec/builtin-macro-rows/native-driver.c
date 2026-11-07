#include <stdbool.h>
#include <stdio.h>
extern unsigned invoked_line(void);
extern unsigned invoked_column(void);
extern bool configured(void);
int main(void) {
    printf("%u %u %d\n", invoked_line(), invoked_column(), (int)configured());
    return 0;
}
