#include <stdint.h>
#include <stdio.h>
uint32_t obs_direct(void),obs_other(void),obs_borrowed(void),obs_generic(void),obs_forwarded(void),obs_index_left(void),obs_index_right(void),obs_index_mut(void),obs_implicit(void);
int main(void) {
    printf("%u %u %u %u %u %u %u %u %u\n",obs_direct(),obs_other(),obs_borrowed(),obs_generic(),obs_forwarded(),obs_index_left(),obs_index_right(),obs_index_mut(),obs_implicit());
    return 0;
}
