#include <stdint.h>
struct Pair { uint32_t a; double b; };
const uint32_t provided_readonly=7;
uint32_t provided_mutable=9;
const double provided_float=1.25;
const struct Pair provided_pair={7,2.5};
static uint32_t plus_three(uint32_t x){return x+3;}
uint32_t (*const provided_callback)(uint32_t)=plus_three;
const uint32_t *const provided_pointer=&provided_readonly;
