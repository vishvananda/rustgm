#include <stdint.h>
#include <stdio.h>
uint32_t obs_direct(void);
uint32_t obs_outer(void);
uint32_t obs_outer_rhs(void);
uint32_t obs_nested(void);
uint32_t obs_nested_rhs(void);
uint32_t obs_generic(void);
uint32_t obs_generic_rhs(void);
uint32_t obs_trait(void);
int main(void) { printf("%u %u %u %u %u %u %u %u\n",obs_direct(),obs_outer(),obs_outer_rhs(),obs_nested(),obs_nested_rhs(),obs_generic(),obs_generic_rhs(),obs_trait()); return 0; }
