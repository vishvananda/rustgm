#include <stdint.h>
#include <stdio.h>
uint32_t obs_wrap_equal(void);
uint32_t obs_wrap_different(void);
uint32_t obs_reference_equal(void);
uint32_t obs_reference_different(void);
uint32_t obs_nested_equal(void);
uint32_t obs_nested_different(void);
uint32_t obs_generic_equal(void);
uint32_t obs_generic_different(void);
int main(void) { printf("%u %u %u %u %u %u %u %u\n",obs_wrap_equal(),obs_wrap_different(),obs_reference_equal(),obs_reference_different(),obs_nested_equal(),obs_nested_different(),obs_generic_equal(),obs_generic_different()); return 0; }
