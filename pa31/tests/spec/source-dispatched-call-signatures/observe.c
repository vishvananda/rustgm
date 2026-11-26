#include <stdint.h>
#include <stdio.h>
uint32_t obs_free(void);
uint32_t obs_map(void);
uint32_t obs_tuple(void);
uint32_t obs_nominal(void);
uint32_t obs_array(void);
uint32_t obs_distinct(void);
uint32_t obs_forward(void);
uint32_t obs_explicit(void);
uint32_t obs_scope(void);
int main(void) { printf("%u %u %u %u %u %u %u %u %u\n",obs_free(),obs_map(),obs_tuple(),obs_nominal(),obs_array(),obs_distinct(),obs_forward(),obs_explicit(),obs_scope()); return 0; }
