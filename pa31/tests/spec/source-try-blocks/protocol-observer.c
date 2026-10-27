#include <stdio.h>
extern unsigned char try_tail(void);
extern unsigned char try_function(void);
extern unsigned char try_local(void);
extern unsigned char try_nested(void);
extern unsigned char try_loop(void);
extern unsigned char try_custom(void);
extern unsigned char try_borrow(void);
int main(void) {
    printf("%u %u %u %u %u %u %u\n", (unsigned)try_tail(),
           (unsigned)try_function(), (unsigned)try_local(), (unsigned)try_nested(),
           (unsigned)try_loop(), (unsigned)try_custom(), (unsigned)try_borrow());
}
