#include <stdint.h>
#include <stdio.h>

uint32_t obs_hrtb_a(void);
uint32_t obs_hrtb_b(void);
uint32_t obs_hrtb_c(void);
uint32_t obs_fn_a(void);
uint32_t obs_fn_b(void);
uint32_t obs_const(void);

int main(void) {
    printf("%u %u %u %u %u %u\n",
           obs_hrtb_a(), obs_hrtb_b(), obs_hrtb_c(),
           obs_fn_a(), obs_fn_b(), obs_const());
    return 0;
}
