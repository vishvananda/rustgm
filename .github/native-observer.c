#include <stdint.h>
#include <stddef.h>
#include <stdio.h>
extern size_t scalar_size(void), array_size(void), pair_size(void), pair_align(void);
extern size_t zero_size(void), zero_align(void);
extern uint8_t clone_value(uint8_t), clone_from_value(uint8_t);
int main(void) {
    const size_t a=scalar_size(), b=array_size(), c=pair_size(), d=pair_align();
    const size_t e=zero_size(), f=zero_align();
    const unsigned g=clone_value(7), h=clone_from_value(9);
    printf("%zu %zu %zu %zu %zu %zu %u %u\n",a,b,c,d,e,f,g,h);
    return a==2 && b==6 && c==4 && d==2 && e==0 && f==8 && g==7 && h==9 ? 0 : 1;
}
