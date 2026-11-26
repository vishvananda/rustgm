#include <stdint.h>
#include <stdio.h>
uint32_t obs_outer(void);
uint32_t obs_borrow(void);
uint32_t obs_nested(void);
uint32_t obs_nested_borrow(void);
uint32_t obs_coerced(void);
uint32_t obs_field(void);
uint32_t obs_indexed(void);
uint32_t obs_direct_index(void);
uint32_t obs_replace_old(void);
uint32_t obs_replace_new(void);
uint32_t obs_control(void);
int main(void) { printf("%u %u %u %u %u %u %u %u %u %u %u\n",obs_outer(),obs_borrow(),obs_nested(),obs_nested_borrow(),obs_coerced(),obs_field(),obs_indexed(),obs_direct_index(),obs_replace_old(),obs_replace_new(),obs_control()); return 0; }
