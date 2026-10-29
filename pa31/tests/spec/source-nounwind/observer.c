#include <stdbool.h>
#include <stdio.h>
extern bool observed_rust(bool);
extern bool observed_c(bool);
extern bool observed_pointer(bool);
int main(void) {
    printf("%d %d %d %d %d %d\n", observed_rust(false), observed_rust(true),
           observed_c(false), observed_c(true), observed_pointer(true), observed_pointer(false));
    return 0;
}
